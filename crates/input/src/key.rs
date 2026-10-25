// Each key once: its variant, its name in chords and the winit key it comes from.
macro_rules! keys {
    ($($variant:ident => $name:literal => $winit:ident,)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Key {
            $($variant,)*
        }

        impl Key {
            pub const ALL: &[Key] = &[$(Key::$variant,)*];

            pub fn name(self) -> &'static str {
                match self {
                    $(Key::$variant => $name,)*
                }
            }

            #[cfg(feature = "winit")]
            pub fn from_winit(key: winit::keyboard::KeyCode) -> Option<Key> {
                match key {
                    $(winit::keyboard::KeyCode::$winit => Some(Key::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

keys! {
    A => "a" => KeyA,
    B => "b" => KeyB,
    C => "c" => KeyC,
    D => "d" => KeyD,
    E => "e" => KeyE,
    F => "f" => KeyF,
    G => "g" => KeyG,
    H => "h" => KeyH,
    I => "i" => KeyI,
    J => "j" => KeyJ,
    K => "k" => KeyK,
    L => "l" => KeyL,
    M => "m" => KeyM,
    N => "n" => KeyN,
    O => "o" => KeyO,
    P => "p" => KeyP,
    Q => "q" => KeyQ,
    R => "r" => KeyR,
    S => "s" => KeyS,
    T => "t" => KeyT,
    U => "u" => KeyU,
    V => "v" => KeyV,
    W => "w" => KeyW,
    X => "x" => KeyX,
    Y => "y" => KeyY,
    Z => "z" => KeyZ,
    Digit0 => "0" => Digit0,
    Digit1 => "1" => Digit1,
    Digit2 => "2" => Digit2,
    Digit3 => "3" => Digit3,
    Digit4 => "4" => Digit4,
    Digit5 => "5" => Digit5,
    Digit6 => "6" => Digit6,
    Digit7 => "7" => Digit7,
    Digit8 => "8" => Digit8,
    Digit9 => "9" => Digit9,
    F1 => "f1" => F1,
    F2 => "f2" => F2,
    F3 => "f3" => F3,
    F4 => "f4" => F4,
    F5 => "f5" => F5,
    F6 => "f6" => F6,
    F7 => "f7" => F7,
    F8 => "f8" => F8,
    F9 => "f9" => F9,
    F10 => "f10" => F10,
    F11 => "f11" => F11,
    F12 => "f12" => F12,
    Backquote => "`" => Backquote,
    Minus => "-" => Minus,
    Equal => "=" => Equal,
    BracketLeft => "[" => BracketLeft,
    BracketRight => "]" => BracketRight,
    Backslash => "\\" => Backslash,
    Semicolon => ";" => Semicolon,
    Quote => "'" => Quote,
    Comma => "," => Comma,
    Period => "." => Period,
    Slash => "/" => Slash,
    Tab => "tab" => Tab,
    Escape => "escape" => Escape,
    Space => "space" => Space,
    Enter => "enter" => Enter,
    Backspace => "backspace" => Backspace,
    Insert => "insert" => Insert,
    Delete => "delete" => Delete,
    Home => "home" => Home,
    End => "end" => End,
    PageUp => "page-up" => PageUp,
    PageDown => "page-down" => PageDown,
    ArrowUp => "up" => ArrowUp,
    ArrowDown => "down" => ArrowDown,
    ArrowLeft => "left" => ArrowLeft,
    ArrowRight => "right" => ArrowRight,
    ShiftLeft => "left-shift" => ShiftLeft,
    ShiftRight => "right-shift" => ShiftRight,
    ControlLeft => "left-ctrl" => ControlLeft,
    ControlRight => "right-ctrl" => ControlRight,
    AltLeft => "left-alt" => AltLeft,
    AltRight => "right-alt" => AltRight,
    SuperLeft => "left-super" => SuperLeft,
    SuperRight => "right-super" => SuperRight,
}

impl Key {
    pub fn from_name(name: &str) -> Option<Key> {
        Key::ALL.iter().copied().find(|key| key.name() == name)
    }
}
