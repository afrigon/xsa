use anyhow::Context;
use usage::Args;
use usage::complete::Shell;

use crate::arguments::Arguments;

#[derive(Args)]
pub struct CompletionArguments {
    #[usage(arg, choices("bash", "elvish", "fish", "nu", "powershell", "zsh"))]
    shell: String,
}

impl CompletionArguments {
    pub fn run(self) -> anyhow::Result<()> {
        let shell = Shell::from_name(&self.shell).context("unsupported shell")?;
        print!("{}", Arguments::completion_script(shell));

        Ok(())
    }
}
