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
