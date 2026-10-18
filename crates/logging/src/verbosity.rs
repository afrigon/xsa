use tracing::Level;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verbosity {
    Quiet,
    Regular,
    Verbose,
    Debug,
    Trace,
}

impl Verbosity {
    pub fn from_flags(verbose: u8, quiet: bool) -> Verbosity {
        match verbose {
            _ if quiet => Verbosity::Quiet,
            0 => Verbosity::Regular,
            1 => Verbosity::Verbose,
            2 => Verbosity::Debug,
            _ => Verbosity::Trace,
        }
    }

    pub fn level(self) -> Level {
        match self {
            Verbosity::Quiet => Level::ERROR,
            Verbosity::Regular => Level::WARN,
            Verbosity::Verbose => Level::INFO,
            Verbosity::Debug => Level::DEBUG,
            Verbosity::Trace => Level::TRACE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_wins_over_verbose() {
        assert_eq!(Verbosity::from_flags(2, true), Verbosity::Quiet);
    }

    #[test]
    fn every_verbose_flag_past_the_third_stays_at_trace() {
        assert_eq!(Verbosity::from_flags(3, false), Verbosity::Trace);
        assert_eq!(Verbosity::from_flags(5, false), Verbosity::Trace);
    }
}
