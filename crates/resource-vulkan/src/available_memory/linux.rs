use std::fs::File;
use std::io::{Read, Seek};

use anyhow::Context;

const MEMORY_INFORMATION: &str = "/proc/meminfo";
const AVAILABLE_FIELD: &str = "MemAvailable:";
const BYTES_PER_KIBIBYTE: u64 = 1024;

// The RAM the system can give processes without swapping, as the kernel estimates it.
pub(crate) struct AvailableMemory {
    file: File,
    text: String,
}

impl AvailableMemory {
    pub fn new() -> anyhow::Result<AvailableMemory> {
        Ok(AvailableMemory {
            file: File::open(MEMORY_INFORMATION).with_context(|| format!("opening {MEMORY_INFORMATION}"))?,
            text: String::new(),
        })
    }

    pub fn bytes(&mut self) -> anyhow::Result<u64> {
        self.text.clear();
        self.file.rewind()?;
        self.file.read_to_string(&mut self.text)?;

        AvailableMemory::parse(&self.text).with_context(|| format!("no {AVAILABLE_FIELD} in {MEMORY_INFORMATION}"))
    }

    // The field reads `MemAvailable:   12345 kB`.
    fn parse(text: &str) -> Option<u64> {
        let line = text.lines().find_map(|line| line.strip_prefix(AVAILABLE_FIELD))?;
        let kibibytes = line.split_whitespace().next()?.parse::<u64>().ok()?;

        Some(kibibytes * BYTES_PER_KIBIBYTE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_available_field() {
        let text = "MemTotal:       32768 kB\nMemFree:         1024 kB\nMemAvailable:    2048 kB\n";

        assert_eq!(AvailableMemory::parse(text), Some(2048 * 1024));
    }

    #[test]
    fn reads_this_machine() {
        assert!(AvailableMemory::new().and_then(|mut memory| memory.bytes()).is_ok());
    }
}
