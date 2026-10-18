use usage::Args;

use crate::verbosity::Verbosity;

#[derive(Args)]
pub struct VerbosityArguments {
    #[usage(short, long, count, global, help = "Log more: -v info, -vv debug, -vvv trace")]
    pub verbose: u8,
    #[usage(short, long, global, help = "Log errors only")]
    pub quiet: bool,
}

impl VerbosityArguments {
    pub fn verbosity(&self) -> Verbosity {
        Verbosity::from_flags(self.verbose, self.quiet)
    }
}
