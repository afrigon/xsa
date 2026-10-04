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
    #[usage(short, long, count, global, help = "Log more: -v info, -vv debug, -vvv trace")]
    pub verbose: u8,
    #[usage(short, long, global, help = "Log errors only")]
    pub quiet: bool,
    #[usage(subcommand)]
    pub command: Command,
}
