use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anstream::adapter::strip_str;
use anstyle::{AnsiColor, Effects, Style};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{oneshot, watch};
use xsa_proto::frame_stream::FrameStream;

use super::IpcTranscript;
use super::ipc_request::IpcRequest;
use super::ipc_response::IpcResponse;
use super::sequence_number::SequenceNumber;
use crate::completion::{CommandCompletion, Completions};
use crate::router::{CommandExecution, CommandInvocation, Output};
use crate::shell::Shell;

const FIRST_SEQUENCE_NUMBER: u64 = 1;

// Every accepted stream is handled by a clone, so all of them share the sequence numbers.
#[derive(Clone)]
pub(super) struct IpcConnection {
    invocations: UnboundedSender<CommandInvocation>,
    transcript: watch::Receiver<IpcTranscript>,
    sequence: Arc<AtomicU64>,
}

impl IpcConnection {
    const REQUEST_STYLE: Style = AnsiColor::BrightCyan.on_default().effects(Effects::BOLD);
    const SUCCESS_STYLE: Style = AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);
    const FAILURE_STYLE: Style = AnsiColor::BrightRed.on_default().effects(Effects::BOLD);
    const OUTPUT_STYLE: Style = Style::new().effects(Effects::DIMMED);

    pub fn new(
        invocations: UnboundedSender<CommandInvocation>,
        transcript: watch::Receiver<IpcTranscript>,
    ) -> IpcConnection {
        IpcConnection {
            invocations,
            transcript,
            sequence: Arc::new(AtomicU64::new(FIRST_SEQUENCE_NUMBER)),
        }
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
        let sequence = SequenceNumber::new(self.sequence.fetch_add(1, Ordering::Relaxed));
        let transcript = *self.transcript.borrow();

        if transcript.shows_input() {
            IpcConnection::print_request(sequence, &words);
        }

        let started = Instant::now();
        let output = self.run(words).await;

        if transcript.shows_output() {
            IpcConnection::print_response(sequence, &output, started.elapsed());
        }

        output
    }

    async fn run(&self, words: Vec<String>) -> Output {
        let (reply, receiver) = oneshot::channel();
        let invocation = CommandInvocation::Execute(CommandExecution {
            words,
            reply,
            styled: false,
        });

        if self.invocations.send(invocation).is_err() {
            return Output::failure("the process is shutting down");
        }

        receiver
            .await
            .unwrap_or_else(|_| Output::failure("the command was dropped before it finished"))
    }

    // The words come from outside the process and the output may quote them, so their escape codes are removed.
    fn print_request(sequence: SequenceNumber, words: &[String]) {
        let style = IpcConnection::REQUEST_STYLE;
        let line = shlex::try_join(words.iter().map(String::as_str)).unwrap_or_else(|_| words.join(" "));
        let line = strip_str(&line);

        let _ = Shell::new().print(&format!("{sequence} {style}▶{style:#} {line}\n"));
    }

    fn print_response(sequence: SequenceNumber, output: &Output, elapsed: Duration) {
        let (style, status) = if output.succeeded {
            (IpcConnection::SUCCESS_STYLE, "ok")
        } else {
            (IpcConnection::FAILURE_STYLE, "failed")
        };
        let gutter = IpcConnection::OUTPUT_STYLE;
        let text = strip_str(&output.text).to_string();
        let body: String = text
            .trim_end()
            .lines()
            .map(|line| format!("  {gutter}│{gutter:#} {line}\n"))
            .collect();

        let _ = Shell::new().print(&format!(
            "{sequence} {style}◀{style:#} {status} in {elapsed:.2?}\n{body}"
        ));
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
