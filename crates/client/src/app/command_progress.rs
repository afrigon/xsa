use xsa_commands::command::ServerCommand;

use super::command_task::CommandTask;

pub(super) enum CommandProgress {
    Done(anyhow::Result<String>),
    Running(Box<dyn CommandTask>),
    // The command ends in a request for the server, replying as the server answers it.
    Forward(Box<dyn ServerCommand>),
}
