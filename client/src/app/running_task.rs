use xsa_commands::router::CommandReply;

use super::command_task::CommandTask;

// Without a reply the task was started by a bind, and its result is logged instead.
pub(super) struct RunningTask {
    pub task: Box<dyn CommandTask>,
    pub reply: Option<CommandReply>,
}
