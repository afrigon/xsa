use usage::Cli;
use xsa_logging::VerbosityArguments;

use crate::cli::commands::Command;

#[derive(Cli)]
#[usage(
    bin = "launchpad",
    version,
    about = "xsa instance and pack manager",
    completion,
    unknown_flags = "error",
    args_override_self = false
)]
pub struct Arguments {
    #[usage(flatten)]
    pub verbosity: VerbosityArguments,
    #[usage(subcommand)]
    pub command: Command,
}
