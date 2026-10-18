use std::fmt::Display;
use std::io::{self, Write};

use anstyle::{AnsiColor, Effects, Style};
use xsa_logging::Verbosity;

pub struct Shell {
    stdout: anstream::Stdout,
    stderr: anstream::Stderr,
    verbosity: Verbosity,
}

impl Shell {
    const STATUS_STYLE: Style = AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);
    const ERROR_STYLE: Style = AnsiColor::BrightRed.on_default().effects(Effects::BOLD);
    const WARNING_STYLE: Style = AnsiColor::BrightYellow.on_default().effects(Effects::BOLD);
    const NOTE_STYLE: Style = AnsiColor::BrightCyan.on_default().effects(Effects::BOLD);
    const STATUS_WIDTH: usize = 12;

    pub fn new(verbosity: Verbosity) -> Shell {
        Shell {
            stdout: anstream::stdout(),
            stderr: anstream::stderr(),
            verbosity,
        }
    }

    pub fn out(&mut self) -> &mut dyn Write {
        &mut self.stdout
    }

    #[expect(dead_code, reason = "for the commands that print progress")]
    pub fn err(&mut self) -> &mut dyn Write {
        &mut self.stderr
    }

    pub fn status(&mut self, status: impl Display, message: impl Display) -> io::Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        let style = Self::STATUS_STYLE;
        let width = Self::STATUS_WIDTH;

        writeln!(self.stderr, "{style}{status:>width$}{style:#} {message}")
    }

    #[expect(dead_code, reason = "for the commands that print progress")]
    pub fn warn(&mut self, message: impl Display) -> io::Result<()> {
        self.labelled("warning", Self::WARNING_STYLE, message)
    }

    #[expect(dead_code, reason = "for the commands that print progress")]
    pub fn note(&mut self, message: impl Display) -> io::Result<()> {
        if self.verbosity == Verbosity::Quiet {
            return Ok(());
        }

        self.labelled("note", Self::NOTE_STYLE, message)
    }

    pub fn error(&mut self, error: &anyhow::Error) -> io::Result<()> {
        self.labelled("error", Self::ERROR_STYLE, error)?;

        for cause in error.chain().skip(1) {
            writeln!(self.stderr, "\nCaused by:")?;

            for line in cause.to_string().lines() {
                writeln!(self.stderr, "  {line}")?;
            }
        }

        Ok(())
    }

    fn labelled(&mut self, label: &str, style: Style, message: impl Display) -> io::Result<()> {
        writeln!(self.stderr, "{style}{label}{style:#}: {message}")
    }
}
