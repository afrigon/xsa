use std::io::{self, Write};
use std::sync::Mutex;

use rustyline::ExternalPrinter;

static PRINTER: Mutex<Option<Box<dyn ExternalPrinter + Send>>> = Mutex::new(None);

// One log event: buffered, then printed above the prompt when the REPL is reading, or to stderr otherwise.
pub struct ConsoleLog {
    buffer: Vec<u8>,
}

impl ConsoleLog {
    pub fn writer() -> ConsoleLog {
        ConsoleLog { buffer: Vec::new() }
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
}

impl Write for ConsoleLog {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(bytes);

        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for ConsoleLog {
    fn drop(&mut self) {
        let text = String::from_utf8_lossy(&self.buffer).into_owned();
        let printed = PRINTER
            .lock()
            .ok()
            .and_then(|mut attached| attached.as_mut().map(|printer| printer.print(text.clone()).is_ok()));

        if printed != Some(true) {
            let _ = io::stderr().write_all(text.as_bytes());
        }
    }
}
