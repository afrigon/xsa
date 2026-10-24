use std::time::Instant;

use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;

use super::{CommandExecution, CommandExecutor, CommandRouter, Output, ScriptProgress};
use crate::script::Script;
use crate::shell::Transcript;

pub(crate) struct RunningScript {
    script: Script,
    next_line: usize,
    pending: Option<PendingLine>,
    reply: Option<oneshot::Sender<Output>>,
}

struct PendingLine {
    location: String,
    started: Instant,
    receiver: oneshot::Receiver<Output>,
}

impl RunningScript {
    pub fn new(script: Script, reply: Option<oneshot::Sender<Output>>) -> RunningScript {
        RunningScript {
            script,
            next_line: 0,
            pending: None,
            reply,
        }
    }

    // Sends lines as long as each one has already replied, so commands that finish at once run back to back.
    pub fn step(
        &mut self,
        router: &mut CommandRouter,
        executor: &mut impl CommandExecutor,
        transcript: Transcript,
    ) -> ScriptProgress {
        let mut progress = ScriptProgress::Waiting;

        loop {
            if let Some(pending) = &mut self.pending {
                let output = match pending.receiver.try_recv() {
                    Ok(output) => output,
                    Err(TryRecvError::Empty) => return progress,
                    Err(TryRecvError::Closed) => Output::failure("the command was dropped before it finished"),
                };
                let pending = self.pending.take().expect("a line is pending");
                transcript.print_response(&pending.location, &output, pending.started.elapsed());

                if !output.succeeded {
                    self.fail(&pending.location, &output.text);

                    return ScriptProgress::Finished;
                }
            }

            let Some(line) = self.script.line(self.next_line) else {
                self.succeed();

                return ScriptProgress::Finished;
            };
            let location = format!("{}:{}", self.script.name(), line.number);
            let words = line.words.clone();
            self.next_line += 1;
            transcript.print_request(&location, &words);
            let (reply, receiver) = oneshot::channel();
            self.pending = Some(PendingLine {
                location,
                started: Instant::now(),
                receiver,
            });
            router.execute(
                CommandExecution {
                    words,
                    reply,
                    styled: false,
                },
                executor,
            );
            progress = ScriptProgress::Progressed;
        }
    }

    fn succeed(&mut self) {
        if let Some(reply) = self.reply.take() {
            let noun = if self.next_line == 1 { "command" } else { "commands" };
            let _ = reply.send(Output::success(format!("script: ran {} {noun}", self.next_line)));
        }
    }

    fn fail(&mut self, location: &str, text: &str) {
        let text = format!("{location}: {}\nskipped the rest of the script", text.trim_end());

        match self.reply.take() {
            Some(reply) => {
                let _ = reply.send(Output::failure(text));
            }
            None => tracing::error!("{text}"),
        }
    }
}
