use xsa_proto::session::ServerSession;

#[cfg(feature = "client")]
use crate::command::ClientCommand;
use crate::completion::{CompletionCandidate, CompletionKind};

pub trait CommandExecutor {
    fn session(&mut self) -> &mut ServerSession;

    fn exit(&mut self);

    fn completion_values(&self, _kind: CompletionKind) -> Vec<CompletionCandidate> {
        Vec::new()
    }

    #[cfg(feature = "client")]
    fn run_client(&mut self, _command: ClientCommand) -> anyhow::Result<String> {
        anyhow::bail!("this command is only available in the game client")
    }
}
