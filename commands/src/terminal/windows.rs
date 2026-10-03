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
