use anyhow::{Context, bail};
use tokio::runtime;
use xsa_proto::network::FrameStream;

use super::ipc_request::IpcRequest;
use super::ipc_response::IpcResponse;
use super::platform;
use crate::completion::Completions;
use crate::router::Output;

pub struct IpcClient {
    name: Option<String>,
}

impl IpcClient {
    pub fn new(name: Option<String>) -> IpcClient {
        IpcClient { name }
    }

    pub fn list() -> anyhow::Result<Vec<String>> {
        IpcClient::block_on(platform::names())
    }

    pub fn send(&self, words: Vec<String>) -> anyhow::Result<Output> {
        match IpcClient::block_on(self.request(IpcRequest::Execute { words }))? {
            IpcResponse::Output { text, succeeded } => Ok(Output { text, succeeded }),
            IpcResponse::Completions { .. } => bail!("the process answered a command with completions"),
        }
    }

    pub fn complete(&self, line: String, cursor: usize) -> anyhow::Result<Completions> {
        match IpcClient::block_on(self.request(IpcRequest::Complete { line, cursor }))? {
            IpcResponse::Completions { completions } => Ok(completions),
            IpcResponse::Output { .. } => bail!("the process answered a completion with command output"),
        }
    }

    async fn request(&self, request: IpcRequest) -> anyhow::Result<IpcResponse> {
        let name = match &self.name {
            Some(name) => name.clone(),
            None => IpcClient::only_name().await?,
        };
        let stream = platform::connect(&name)
            .await
            .with_context(|| format!("no xsa process named {name} is running"))?;
        let mut stream = FrameStream::new(stream);
        stream.write(&request).await?;

        stream
            .read()
            .await?
            .with_context(|| format!("process {name} closed the connection without answering"))
    }

    async fn only_name() -> anyhow::Result<String> {
        let mut names = platform::names().await?;

        match names.len() {
            0 => bail!("no xsa process is running"),
            1 => Ok(names.remove(0)),
            _ => bail!(
                "several xsa processes are running, pick one with --name: {}",
                names.join(", ")
            ),
        }
    }

    fn block_on<T>(future: impl Future<Output = anyhow::Result<T>>) -> anyhow::Result<T> {
        runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(future)
    }
}
