mod completion_command;

pub use completion_command::CompletionCommand;
use usage::Subcommands;

use crate::cli::CommandHandler;
use crate::launchpad_context::LaunchpadContext;

#[derive(Subcommands)]
pub enum Command {
    /// Print the shell completion script, e.g. launchpad completion fish | source
    Completion(CompletionCommand),
}

impl CommandHandler for Command {
    async fn handle(&self, context: &mut LaunchpadContext) -> anyhow::Result<()> {
        match self {
            Command::Completion(command) => command.handle(context).await,
        }
    }
}
