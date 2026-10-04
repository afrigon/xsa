use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use xsa_proto::frame_stream::FrameStream;

use super::ipc_request::IpcRequest;
use super::ipc_response::IpcResponse;
use crate::completion::{CommandCompletion, Completions};
use crate::router::{CommandExecution, CommandInvocation, Output};

pub(super) struct IpcConnection {
    invocations: UnboundedSender<CommandInvocation>,
}

impl IpcConnection {
    pub fn new(invocations: UnboundedSender<CommandInvocation>) -> IpcConnection {
        IpcConnection { invocations }
    }

    pub async fn handle(self, stream: impl AsyncRead + AsyncWrite + Unpin) {
        let mut stream = FrameStream::new(stream);

        while let Ok(Some(request)) = stream.read::<IpcRequest>().await {
            let response = match request {
                IpcRequest::Execute { words } => {
                    let output = self.execute(words).await;

                    IpcResponse::Output {
                        text: output.text,
                        succeeded: output.succeeded,
                    }
                }
                IpcRequest::Complete { line, cursor } => IpcResponse::Completions {
                    completions: self.complete(line, cursor).await,
                },
            };

            if stream.write(&response).await.is_err() {
                return;
            }
        }
    }

    async fn execute(&self, words: Vec<String>) -> Output {
        let (reply, receiver) = oneshot::channel();
        let invocation = CommandInvocation::Execute(CommandExecution {
            words,
            reply,
            styled: false,
        });

        if self.invocations.send(invocation).is_err() {
            return Output::failure("the instance is shutting down");
        }

        receiver
            .await
            .unwrap_or_else(|_| Output::failure("the command was dropped before it finished"))
    }

    async fn complete(&self, line: String, cursor: usize) -> Completions {
        let (reply, receiver) = oneshot::channel();
        let completion = CommandCompletion { line, cursor, reply };

        if self.invocations.send(CommandInvocation::Complete(completion)).is_err() {
            return Completions::default();
        }

        receiver.await.unwrap_or_default()
    }
}
