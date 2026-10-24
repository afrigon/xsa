mod shell_log;
mod transcript;

pub use shell_log::ShellLog;
pub use transcript::Transcript;

use std::io::{self, Write};
use std::sync::Mutex;

use anstream::ColorChoice;
use anstream::adapter::strip_str;
use rustyline::ExternalPrinter;

static PRINTER: Mutex<Option<Box<dyn ExternalPrinter + Send>>> = Mutex::new(None);

// While the REPL is reading, text goes through its printer so it lands above the prompt instead of through it.
pub struct Shell {
    stdout: anstream::Stdout,
    stderr: anstream::Stderr,
}

impl Shell {
    pub fn new() -> Shell {
        Shell {
            stdout: anstream::stdout(),
            stderr: anstream::stderr(),
        }
    }

    pub fn print(&mut self, text: &str) -> io::Result<()> {
        if Shell::print_above_prompt(text, self.stdout.current_choice()) {
            return Ok(());
        }

        self.stdout.write_all(text.as_bytes())
    }

    pub fn log(&mut self, text: &str) -> io::Result<()> {
        if Shell::print_above_prompt(text, self.stderr.current_choice()) {
            return Ok(());
        }

        self.stderr.write_all(text.as_bytes())
    }

    pub(crate) fn attach(printer: impl ExternalPrinter + Send + 'static) {
        if let Ok(mut attached) = PRINTER.lock() {
            *attached = Some(Box::new(printer));
        }
    }

    pub(crate) fn detach() {
        if let Ok(mut attached) = PRINTER.lock() {
            *attached = None;
        }
    }

    fn print_above_prompt(text: &str, choice: ColorChoice) -> bool {
        let text = match choice {
            ColorChoice::Never => strip_str(text).to_string(),
            _ => text.to_string(),
        };
        let printed = PRINTER
            .lock()
            .ok()
            .and_then(|mut attached| attached.as_mut().map(|printer| printer.print(text).is_ok()));

        printed == Some(true)
    }
}

impl Default for Shell {
    fn default() -> Self {
        Shell::new()
    }
}
