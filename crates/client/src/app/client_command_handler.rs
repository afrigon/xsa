use super::App;
use super::command_progress::CommandProgress;

pub(super) trait ClientCommandHandler {
    fn start(self, app: &mut App) -> CommandProgress;
}
