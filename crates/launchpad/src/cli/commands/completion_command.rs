use anyhow::Context;
use usage::Args;
use usage::complete;

use crate::cli::{Arguments, CommandHandler};
use crate::launchpad_context::LaunchpadContext;

#[derive(Args)]
pub struct CompletionCommand {
    #[usage(arg, choices("bash", "elvish", "fish", "nu", "powershell", "zsh"))]
    shell: String,
}

impl CommandHandler for CompletionCommand {
    async fn handle(&self, context: &mut LaunchpadContext) -> anyhow::Result<()> {
        let shell = complete::Shell::from_name(&self.shell).context("unsupported shell")?;
        write!(context.shell().out(), "{}", Arguments::completion_script(shell))?;

        Ok(())
    }
}
