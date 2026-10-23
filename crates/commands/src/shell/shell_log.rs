use std::io::{self, Write};

use super::Shell;

// One log event, buffered so it reaches the shell whole.
pub struct ShellLog {
    buffer: Vec<u8>,
}

impl ShellLog {
    pub fn writer() -> ShellLog {
        ShellLog { buffer: Vec::new() }
    }
}

impl Write for ShellLog {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(bytes);

        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for ShellLog {
    fn drop(&mut self) {
        let _ = Shell::new().log(&String::from_utf8_lossy(&self.buffer));
    }
}
