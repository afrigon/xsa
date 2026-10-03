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

#[cfg(unix)]
mod platform {
    use std::io::{self, IsTerminal, Write};

    use nix::sys::termios::{self, SetArg, Termios};

    const BRACKETED_PASTE_OFF: &[u8] = b"\x1b[?2004l";

    pub struct SavedMode {
        termios: Termios,
    }

    pub fn capture() -> Option<SavedMode> {
        let stdin = io::stdin();
        if !stdin.is_terminal() {
            return None;
        }
        termios::tcgetattr(&stdin).ok().map(|termios| SavedMode { termios })
    }

    pub fn restore(saved: &SavedMode) {
        let _ = termios::tcsetattr(io::stdin(), SetArg::TCSANOW, &saved.termios);
        let mut stdout = io::stdout();
        let _ = stdout.write_all(BRACKETED_PASTE_OFF);
        let _ = stdout.flush();
    }
}

#[cfg(windows)]
mod platform {
    use windows_sys::Win32::System::Console::{
        CONSOLE_MODE, GetConsoleMode, GetStdHandle, STD_INPUT_HANDLE, SetConsoleMode,
    };

    pub struct SavedMode {
        mode: CONSOLE_MODE,
    }

    pub fn capture() -> Option<SavedMode> {
        let mut mode: CONSOLE_MODE = 0;
        // SAFETY: the standard input handle is owned by the process; GetConsoleMode only writes `mode`.
        let captured = unsafe { GetConsoleMode(GetStdHandle(STD_INPUT_HANDLE), &mut mode) };
        (captured != 0).then_some(SavedMode { mode })
    }

    pub fn restore(saved: &SavedMode) {
        // SAFETY: as above; SetConsoleMode only reads its arguments.
        unsafe {
            SetConsoleMode(GetStdHandle(STD_INPUT_HANDLE), saved.mode);
        }
    }
}
