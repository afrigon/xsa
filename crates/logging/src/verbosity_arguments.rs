use usage::Args;

use crate::LogFilter;
use crate::verbosity::Verbosity;

#[derive(Args)]
pub struct VerbosityArguments {
    #[usage(short, long, count, global, help = "Log more: -v info, -vv debug, -vvv trace")]
    pub verbose: u8,
    #[usage(short, long, global, help = "Log errors only")]
    pub quiet: bool,
}

impl VerbosityArguments {
    pub fn verbosity(&self) -> Option<Verbosity> {
        Verbosity::from_flags(self.verbose, self.quiet)
    }

    // A flag wins over RUST_LOG; None leaves the filter to the config.
    pub fn filter_override(&self) -> Option<LogFilter> {
        self.verbosity()
            .map(Verbosity::filter)
            .or_else(LogFilter::from_environment)
    }
}
