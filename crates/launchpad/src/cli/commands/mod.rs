mod completion_command;
mod terrain;

pub use completion_command::CompletionCommand;
pub use terrain::TerrainCommand;
use usage::Subcommands;

use crate::cli::CommandHandler;
use crate::launchpad_context::LaunchpadContext;

#[derive(Subcommands)]
pub enum Command {
    /// Print the shell completion script, e.g. launchpad completion fish | source
    Completion(CompletionCommand),
    /// Convert terrain layers into the files the game streams
    Terrain(TerrainCommand),
}

impl CommandHandler for Command {
    async fn handle(&self, context: &mut LaunchpadContext) -> anyhow::Result<()> {
        match self {
            Command::Completion(command) => command.handle(context).await,
            Command::Terrain(command) => command.handle(context).await,
        }
    }
}
