use std::env;
use std::fs::{self, DirBuilder};
use std::io::ErrorKind;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc::UnboundedSender;

use super::ipc_connection::IpcConnection;
use super::{Bound, IpcEndpoint};
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
        tokio::spawn(IpcConnection::new(invocations.clone()).handle(stream));
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
