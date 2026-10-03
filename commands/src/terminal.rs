#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
use unix as platform;
#[cfg(windows)]
use windows as platform;

// The prompt puts the terminal in raw mode while it reads; if the process exits mid-read, nothing switches it back.
pub struct TerminalGuard {
    saved: platform::SavedMode,
}

impl TerminalGuard {
    pub fn capture() -> Option<Self> {
        platform::capture().map(|saved| Self { saved })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        platform::restore(&self.saved);
    }
}
