use std::time::Instant;

use xsa_proto::session::ServerSession;

#[cfg(feature = "client")]
use crate::command::ClientCommand;
use crate::command::LocalCommand;
use crate::completion::{CompletionCandidate, CompletionKind};
#[cfg(feature = "client")]
use crate::router::CommandReply;
use crate::router::Output;

pub trait CommandExecutor {
    fn session(&mut self) -> &mut ServerSession;

    fn exit(&mut self);

    fn run_local(&mut self, command: LocalCommand) -> Output;

    // The moment waits are measured against: the wall clock, unless the process paces its own time.
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn completion_values(&self, _kind: CompletionKind) -> Vec<CompletionCandidate> {
        Vec::new()
    }

    // The executor replies when the command finishes, which may be frames later.
    #[cfg(feature = "client")]
    fn run_client(&mut self, _command: ClientCommand, reply: CommandReply) {
        reply.send(Err(anyhow::anyhow!(
            "this command is only available in the game client"
        )));
    }
}
