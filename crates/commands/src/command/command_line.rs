use usage::Cli;

use super::Command;

#[derive(Cli)]
#[usage(bin = "", unknown_flags = "error", args_override_self = false)]
pub struct CommandLine {
    #[usage(subcommand)]
    pub command: Command,
}
