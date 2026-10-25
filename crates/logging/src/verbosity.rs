use tracing::Level;

use crate::LogFilter;
use xsa_config::ConfigValue;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verbosity {
    Quiet,
    Verbose,
    Debug,
    Trace,
}

impl Verbosity {
    // None when no flag is given, so the configured filter applies.
    pub fn from_flags(verbose: u8, quiet: bool) -> Option<Verbosity> {
        match verbose {
            _ if quiet => Some(Verbosity::Quiet),
            0 => None,
            1 => Some(Verbosity::Verbose),
            2 => Some(Verbosity::Debug),
            _ => Some(Verbosity::Trace),
        }
    }

    pub fn level(self) -> Level {
        match self {
            Verbosity::Quiet => Level::ERROR,
            Verbosity::Verbose => Level::INFO,
            Verbosity::Debug => Level::DEBUG,
            Verbosity::Trace => Level::TRACE,
        }
    }

    pub fn filter(self) -> LogFilter {
        LogFilter::parse(&self.level().as_str().to_lowercase()).expect("a level is a valid log filter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_wins_over_verbose() {
        assert_eq!(Verbosity::from_flags(2, true), Some(Verbosity::Quiet));
    }

    #[test]
    fn no_flag_leaves_the_filter_to_the_config() {
        assert_eq!(Verbosity::from_flags(0, false), None);
    }

    #[test]
    fn every_verbose_flag_past_the_third_stays_at_trace() {
        assert_eq!(Verbosity::from_flags(3, false), Some(Verbosity::Trace));
        assert_eq!(Verbosity::from_flags(5, false), Some(Verbosity::Trace));
    }
}
