use xsa_proto::session::ServerSession;

#[cfg(feature = "client")]
use crate::command::ClientCommand;
use crate::completion::{CompletionCandidate, CompletionKind};
#[cfg(feature = "client")]
use crate::router::CommandReply;

pub trait CommandExecutor {
    fn session(&mut self) -> &mut ServerSession;

    fn exit(&mut self);

    // Whether the game is being played, with no menu open; commands available only in game are refused otherwise. An
    // executor without menus, like the dedicated server, always is.
    fn is_in_game(&self) -> bool {
        true
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
