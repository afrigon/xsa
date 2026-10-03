use usage::Cli;

use crate::command::Command;

#[derive(Cli)]
#[usage(
    bin = "xsa",
    version,
    about = "xsa space flight simulator",
    completion,
    unknown_flags = "error",
    args_override_self = false
)]
pub struct Arguments {
    #[usage(subcommand)]
    pub command: Command,
}
