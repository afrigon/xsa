use rustyline::completion::{Completer, Pair};
use rustyline::{Context, Helper, Highlighter, Hinter, Validator};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use xsa_packs::NAMESPACE_SEPARATOR;

use crate::completion::CommandCompletion;
use crate::router::CommandInvocation;

#[derive(Helper, Hinter, Highlighter, Validator)]
pub(super) struct CommandCompleter {
    pub invocations: UnboundedSender<CommandInvocation>,
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, position: usize, _context: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (reply, receiver) = oneshot::channel();
        let completion = CommandCompletion {
            line: line.to_string(),
            cursor: position,
            reply,
        };

        if self.invocations.send(CommandInvocation::Complete(completion)).is_err() {
            return Ok((position, Vec::new()));
        }

        let Ok(completions) = receiver.blocking_recv() else {
            return Ok((position, Vec::new()));
        };
        let candidates = completions
            .candidates
            .into_iter()
            .map(|candidate| Pair {
                display: candidate.value.clone(),
                replacement: if candidate.value.ends_with(NAMESPACE_SEPARATOR) {
                    candidate.value
                } else {
                    format!("{} ", candidate.value)
                },
            })
            .collect();

        Ok((completions.start, candidates))
    }
}
