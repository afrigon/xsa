use xsa_commands::command::ServerCommand;
use xsa_commands::router::CommandReply;

// A request a client command worked out, waiting for the router to send it.
pub(super) struct ForwardedCommand {
    pub command: Box<dyn ServerCommand>,
    pub reply: Option<CommandReply>,
}
