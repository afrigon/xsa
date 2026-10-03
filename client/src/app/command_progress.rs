use super::command_task::CommandTask;

pub(super) enum CommandProgress {
    Done(anyhow::Result<String>),
    Running(Box<dyn CommandTask>),
}
