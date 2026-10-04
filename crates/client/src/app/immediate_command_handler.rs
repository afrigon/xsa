use super::App;
use super::client_command_handler::ClientCommandHandler;
use super::command_progress::CommandProgress;

// A command that finishes within the frame it starts in.
pub(super) trait ImmediateCommandHandler {
    fn run(self, app: &mut App) -> anyhow::Result<String>;
}

impl<Command: ImmediateCommandHandler> ClientCommandHandler for Command {
    fn start(self, app: &mut App) -> CommandProgress {
        CommandProgress::Done(self.run(app))
    }
}
