use std::env;

use anyhow::Context;
use tracing_subscriber::EnvFilter;
use xsa_config::{ConfigValue, KdlValue};

const ENVIRONMENT_VARIABLE: &str = "RUST_LOG";
const DEFAULT_DIRECTIVES: &str = "warn";
const OFF_DIRECTIVES: &str = "off";
const LEVELS: [&str; 6] = ["off", "error", "warn", "info", "debug", "trace"];

// A level, or per-crate directives in RUST_LOG's syntax such as `warn,xsa_client=debug`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogFilter {
    directives: String,
}

impl LogFilter {
    pub fn off() -> LogFilter {
        LogFilter {
            directives: OFF_DIRECTIVES.to_string(),
        }
    }

    pub fn from_environment() -> Option<LogFilter> {
        let directives = env::var(ENVIRONMENT_VARIABLE).ok()?;

        // Logging is not installed yet when this is read, so a bad value can only be printed.
        match LogFilter::parse(&directives) {
            Ok(filter) => Some(filter),
            Err(err) => {
                eprintln!("{ENVIRONMENT_VARIABLE} is ignored: {err:#}");
                None
            }
        }
    }

    pub(crate) fn env_filter(&self) -> EnvFilter {
        EnvFilter::builder()
            .parse(&self.directives)
            .expect("a log filter's directives were checked when it was made")
    }
}

impl Default for LogFilter {
    fn default() -> LogFilter {
        LogFilter {
            directives: DEFAULT_DIRECTIVES.to_string(),
        }
    }
}

impl ConfigValue for LogFilter {
    fn from_kdl(value: &KdlValue) -> Option<LogFilter> {
        value.as_string().and_then(|text| LogFilter::parse(text).ok())
    }

    fn to_kdl(&self) -> KdlValue {
        KdlValue::String(self.directives.clone())
    }

    fn parse(text: &str) -> anyhow::Result<LogFilter> {
        EnvFilter::builder()
            .parse(text)
            .with_context(|| format!("{text:?} is not a log level or filter like warn or warn,xsa_client=debug"))?;

        Ok(LogFilter {
            directives: text.to_string(),
        })
    }

    fn type_name() -> String {
        "log-filter".to_string()
    }

    fn completions() -> Vec<String> {
        LEVELS.iter().map(|level| level.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_level_or_per_crate_directives() {
        assert!(LogFilter::parse("debug").is_ok());
        assert!(LogFilter::parse("warn,xsa_client=debug").is_ok());
        assert!(LogFilter::parse("warn,xsa_client=loud").is_err());
    }
}
