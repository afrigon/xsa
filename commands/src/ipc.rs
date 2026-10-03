#[cfg(unix)]
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::thread;

use anyhow::{Context, bail};
use bitcode::{Decode, Encode};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::runtime;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use xsa_proto::frame::{read_frame, write_frame};

use crate::completion::{CommandCompletion, Completions};
use crate::router::{CommandExecution, CommandInvocation, Output};

#[derive(Encode, Decode)]
enum IpcRequest {
    Execute { words: Vec<String> },
    Complete { line: String, cursor: usize },
}

#[derive(Encode, Decode)]
enum IpcResponse {
    Output { text: String, succeeded: bool },
    Completions { completions: Completions },
}

#[derive(Clone, Copy)]
pub enum InstanceKind {
    Client,
    Server,
}

impl InstanceKind {
    fn default_name(self) -> String {
        let kind = match self {
            InstanceKind::Client => "client",
            InstanceKind::Server => "server",
        };
        format!("{kind}-{}", std::process::id())
    }
}

pub struct IpcEndpoint {
    #[cfg(unix)]
    path: PathBuf,
}

#[cfg(unix)]
impl Drop for IpcEndpoint {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn spawn(
    name: Option<String>,
    kind: InstanceKind,
    invocations: UnboundedSender<CommandInvocation>,
) -> anyhow::Result<IpcEndpoint> {
    let name = name.unwrap_or_else(|| kind.default_name());
    validate_name(&name)?;
    let (bound_sender, bound) = std_mpsc::sync_channel(1);
    thread::Builder::new().name("ipc".into()).spawn(move || {
        let runtime = match runtime::Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => runtime,
            Err(err) => {
                let _ = bound_sender.send(Err(err.into()));
                return;
            }
        };
        runtime.block_on(platform::serve(name, invocations, bound_sender));
    })?;
    bound.recv().context("the IPC thread stopped before listening")?
}

pub fn send(instance: Option<&str>, words: Vec<String>) -> anyhow::Result<Output> {
    match block_on(request(instance, IpcRequest::Execute { words }))? {
        IpcResponse::Output { text, succeeded } => Ok(Output { text, succeeded }),
        IpcResponse::Completions { .. } => bail!("the instance answered a command with completions"),
    }
}

pub fn complete(instance: Option<&str>, line: String, cursor: usize) -> anyhow::Result<Completions> {
    match block_on(request(instance, IpcRequest::Complete { line, cursor }))? {
        IpcResponse::Completions { completions } => Ok(completions),
        IpcResponse::Output { .. } => bail!("the instance answered a completion with command output"),
    }
}

async fn request(instance: Option<&str>, request: IpcRequest) -> anyhow::Result<IpcResponse> {
    let name = match instance {
        Some(name) => name.to_string(),
        None => only_instance().await?,
    };
    let mut stream = platform::connect(&name)
        .await
        .with_context(|| format!("no xsa instance named {name} is running"))?;
    write_frame(&mut stream, &request).await?;
    read_frame(&mut stream)
        .await?
        .with_context(|| format!("instance {name} closed the connection without answering"))
}

pub fn list() -> anyhow::Result<Vec<String>> {
    block_on(platform::instances())
}

fn block_on<T>(future: impl Future<Output = anyhow::Result<T>>) -> anyhow::Result<T> {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(future)
}

async fn only_instance() -> anyhow::Result<String> {
    let mut instances = platform::instances().await?;
    match instances.len() {
        0 => bail!("no xsa instance is running"),
        1 => Ok(instances.remove(0)),
        _ => bail!(
            "several xsa instances are running, pick one with --instance: {}",
            instances.join(", ")
        ),
    }
}

fn validate_name(name: &str) -> anyhow::Result<()> {
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_');
    if !valid {
        bail!("{name:?} is not an instance name: use letters, digits, - and _");
    }
    Ok(())
}

async fn handle(mut stream: impl AsyncRead + AsyncWrite + Unpin, invocations: UnboundedSender<CommandInvocation>) {
    while let Ok(Some(request)) = read_frame::<IpcRequest>(&mut stream).await {
        let response = match request {
            IpcRequest::Execute { words } => {
                let output = execute(words, &invocations).await;
                IpcResponse::Output {
                    text: output.text,
                    succeeded: output.succeeded,
                }
            }
            IpcRequest::Complete { line, cursor } => IpcResponse::Completions {
                completions: complete_here(line, cursor, &invocations).await,
            },
        };
        if write_frame(&mut stream, &response).await.is_err() {
            return;
        }
    }
}

async fn execute(words: Vec<String>, invocations: &UnboundedSender<CommandInvocation>) -> Output {
    let (reply, receiver) = oneshot::channel();
    let invocation = CommandInvocation::Execute(CommandExecution {
        words,
        reply,
        styled: false,
    });
    if invocations.send(invocation).is_err() {
        return Output::failure("the instance is shutting down");
    }
    receiver
        .await
        .unwrap_or_else(|_| Output::failure("the command was dropped before it finished"))
}

async fn complete_here(line: String, cursor: usize, invocations: &UnboundedSender<CommandInvocation>) -> Completions {
    let (reply, receiver) = oneshot::channel();
    let completion = CommandCompletion { line, cursor, reply };
    if invocations.send(CommandInvocation::Complete(completion)).is_err() {
        return Completions::default();
    }
    receiver.await.unwrap_or_default()
}

type Bound = std_mpsc::SyncSender<anyhow::Result<IpcEndpoint>>;

#[cfg(unix)]
mod platform {
    use std::env;
    use std::fs::{self, DirBuilder};
    use std::io::ErrorKind;
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    use std::path::{Path, PathBuf};

    use anyhow::{Context, bail};
    use tokio::net::{UnixListener, UnixStream};
    use tokio::sync::mpsc::UnboundedSender;

    use super::{Bound, IpcEndpoint, handle};
    use crate::router::CommandInvocation;

    const DIRECTORY_NAME: &str = "xsa";
    const SOCKET_EXTENSION: &str = "sock";
    const OWNER_ONLY: u32 = 0o700;

    fn directory() -> PathBuf {
        match env::var_os("XDG_RUNTIME_DIR") {
            Some(runtime) => Path::new(&runtime).join(DIRECTORY_NAME),
            None => {
                let user = env::var("USER").unwrap_or_default();
                env::temp_dir().join(format!("{DIRECTORY_NAME}-{user}"))
            }
        }
    }

    fn socket_path(name: &str) -> PathBuf {
        directory().join(name).with_extension(SOCKET_EXTENSION)
    }

    pub async fn serve(name: String, invocations: UnboundedSender<CommandInvocation>, bound: Bound) {
        let listener = match bind(&name).await {
            Ok(listener) => listener,
            Err(err) => {
                let _ = bound.send(Err(err));
                return;
            }
        };
        let _ = bound.send(Ok(IpcEndpoint {
            path: socket_path(&name),
        }));
        while let Ok((stream, _address)) = listener.accept().await {
            tokio::spawn(handle(stream, invocations.clone()));
        }
    }

    async fn bind(name: &str) -> anyhow::Result<UnixListener> {
        let directory = directory();
        DirBuilder::new()
            .recursive(true)
            .mode(OWNER_ONLY)
            .create(&directory)
            .with_context(|| format!("creating {}", directory.display()))?;
        fs::set_permissions(&directory, fs::Permissions::from_mode(OWNER_ONLY))?;
        let path = socket_path(name);
        if path.exists() {
            if UnixStream::connect(&path).await.is_ok() {
                bail!("an xsa instance named {name} is already running");
            }
            fs::remove_file(&path)?;
        }
        UnixListener::bind(&path).with_context(|| format!("listening on {}", path.display()))
    }

    pub async fn connect(name: &str) -> std::io::Result<UnixStream> {
        UnixStream::connect(socket_path(name)).await
    }

    pub async fn instances() -> anyhow::Result<Vec<String>> {
        let entries = match fs::read_dir(directory()) {
            Ok(entries) => entries,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err.into()),
        };
        let mut names = Vec::new();
        for entry in entries {
            let path = entry?.path();
            if path.extension().is_none_or(|extension| extension != SOCKET_EXTENSION) {
                continue;
            }
            let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            match UnixStream::connect(&path).await {
                Ok(_) => names.push(name.to_string()),
                Err(err) if err.kind() == ErrorKind::ConnectionRefused => {
                    let _ = fs::remove_file(&path);
                }
                Err(_) => {}
            }
        }
        names.sort();
        Ok(names)
    }
}

#[cfg(windows)]
mod platform {
    use std::fs;

    use anyhow::Context;
    use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient, ServerOptions};
    use tokio::sync::mpsc::UnboundedSender;

    use super::{Bound, IpcEndpoint, handle};
    use crate::router::CommandInvocation;

    const PIPE_DIRECTORY: &str = r"\\.\pipe\";
    const PIPE_PREFIX: &str = "xsa-";

    fn pipe_name(name: &str) -> String {
        format!("{PIPE_DIRECTORY}{PIPE_PREFIX}{name}")
    }

    pub async fn serve(name: String, invocations: UnboundedSender<CommandInvocation>, bound: Bound) {
        let path = pipe_name(&name);
        let mut server = match ServerOptions::new()
            .first_pipe_instance(true)
            .create(&path)
            .with_context(|| format!("an xsa instance named {name} is already running"))
        {
            Ok(server) => server,
            Err(err) => {
                let _ = bound.send(Err(err));
                return;
            }
        };
        let _ = bound.send(Ok(IpcEndpoint {}));
        loop {
            if server.connect().await.is_err() {
                return;
            }
            let connected = server;
            server = match ServerOptions::new().create(&path) {
                Ok(server) => server,
                Err(_) => return,
            };
            tokio::spawn(handle(connected, invocations.clone()));
        }
    }

    pub async fn connect(name: &str) -> std::io::Result<NamedPipeClient> {
        ClientOptions::new().open(pipe_name(name))
    }

    pub async fn instances() -> anyhow::Result<Vec<String>> {
        let mut names: Vec<String> = fs::read_dir(PIPE_DIRECTORY)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter_map(|name| name.strip_prefix(PIPE_PREFIX).map(str::to_string))
            .collect();
        names.sort();
        Ok(names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_names_are_restricted() {
        assert!(validate_name("client-1234").is_ok());
        assert!(validate_name("agent_1").is_ok());
        for name in ["", "a/b", "../x", "a b", "a.sock"] {
            assert!(validate_name(name).is_err(), "{name:?}");
        }
    }
}
