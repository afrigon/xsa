use std::fmt::Display;
use std::time::Duration;

use anstream::adapter::strip_str;
use anstyle::{AnsiColor, Effects, Style};
use xsa_config::{ConfigSection, ConfigVisitor};

use super::Shell;
use crate::router::Output;

// Which commands received from outside the shell, and their replies, are printed to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transcript {
    pub enabled: bool,
    pub input: bool,
    pub output: bool,
}

impl Transcript {
    const REQUEST_STYLE: Style = AnsiColor::BrightCyan.on_default().effects(Effects::BOLD);
    const SUCCESS_STYLE: Style = AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);
    const FAILURE_STYLE: Style = AnsiColor::BrightRed.on_default().effects(Effects::BOLD);
    const OUTPUT_STYLE: Style = Style::new().effects(Effects::DIMMED);

    pub fn within(self, enabled: bool) -> Transcript {
        Transcript {
            enabled: self.enabled && enabled,
            ..self
        }
    }

    // The words may come from outside the process and the output may quote them, so their escape codes are removed.
    pub fn print_request(self, label: impl Display, words: &[String]) {
        if !(self.enabled && self.input) {
            return;
        }

        let style = Transcript::REQUEST_STYLE;
        let line = shlex::try_join(words.iter().map(String::as_str)).unwrap_or_else(|_| words.join(" "));
        let line = strip_str(&line);

        let _ = Shell::new().print(&format!("{label} {style}▶{style:#} {line}\n"));
    }

    pub fn print_response(self, label: impl Display, output: &Output, elapsed: Duration) {
        if !(self.enabled && self.output) {
            return;
        }

        let (style, status) = if output.succeeded {
            (Transcript::SUCCESS_STYLE, "ok")
        } else {
            (Transcript::FAILURE_STYLE, "failed")
        };
        let gutter = Transcript::OUTPUT_STYLE;
        let text = strip_str(&output.text).to_string();
        let body: String = text
            .trim_end()
            .lines()
            .map(|line| format!("  {gutter}│{gutter:#} {line}\n"))
            .collect();

        let _ = Shell::new().print(&format!("{label} {style}◀{style:#} {status} in {elapsed:.2?}\n{body}"));
    }
}

impl Default for Transcript {
    fn default() -> Self {
        Transcript {
            enabled: true,
            input: true,
            output: true,
        }
    }
}

impl ConfigSection for Transcript {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
        visitor.group("input", |visitor| visitor.value("enabled", &mut self.input));
        visitor.group("output", |visitor| visitor.value("enabled", &mut self.output));
    }
}
