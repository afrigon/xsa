use winit::keyboard::KeyCode;

use crate::config::KeyChord;

const MODIFIER_SEPARATOR: char = '+';
const SHIFT: &str = "shift";
const CONTROL: &str = "ctrl";
const ALT: &str = "alt";

struct KeyName {
    name: &'static str,
    key: KeyCode,
}

const KEY_NAMES: [KeyName; 74] = [
    KeyName {
        name: "a",
        key: KeyCode::KeyA,
    },
    KeyName {
        name: "b",
        key: KeyCode::KeyB,
    },
    KeyName {
        name: "c",
        key: KeyCode::KeyC,
    },
    KeyName {
        name: "d",
        key: KeyCode::KeyD,
    },
    KeyName {
        name: "e",
        key: KeyCode::KeyE,
    },
    KeyName {
        name: "f",
        key: KeyCode::KeyF,
    },
    KeyName {
        name: "g",
        key: KeyCode::KeyG,
    },
    KeyName {
        name: "h",
        key: KeyCode::KeyH,
    },
    KeyName {
        name: "i",
        key: KeyCode::KeyI,
    },
    KeyName {
        name: "j",
        key: KeyCode::KeyJ,
    },
    KeyName {
        name: "k",
        key: KeyCode::KeyK,
    },
    KeyName {
        name: "l",
        key: KeyCode::KeyL,
    },
    KeyName {
        name: "m",
        key: KeyCode::KeyM,
    },
    KeyName {
        name: "n",
        key: KeyCode::KeyN,
    },
    KeyName {
        name: "o",
        key: KeyCode::KeyO,
    },
    KeyName {
        name: "p",
        key: KeyCode::KeyP,
    },
    KeyName {
        name: "q",
        key: KeyCode::KeyQ,
    },
    KeyName {
        name: "r",
        key: KeyCode::KeyR,
    },
    KeyName {
        name: "s",
        key: KeyCode::KeyS,
    },
    KeyName {
        name: "t",
        key: KeyCode::KeyT,
    },
    KeyName {
        name: "u",
        key: KeyCode::KeyU,
    },
    KeyName {
        name: "v",
        key: KeyCode::KeyV,
    },
    KeyName {
        name: "w",
        key: KeyCode::KeyW,
    },
    KeyName {
        name: "x",
        key: KeyCode::KeyX,
    },
    KeyName {
        name: "y",
        key: KeyCode::KeyY,
    },
    KeyName {
        name: "z",
        key: KeyCode::KeyZ,
    },
    KeyName {
        name: "0",
        key: KeyCode::Digit0,
    },
    KeyName {
        name: "1",
        key: KeyCode::Digit1,
    },
    KeyName {
        name: "2",
        key: KeyCode::Digit2,
    },
    KeyName {
        name: "3",
        key: KeyCode::Digit3,
    },
    KeyName {
        name: "4",
        key: KeyCode::Digit4,
    },
    KeyName {
        name: "5",
        key: KeyCode::Digit5,
    },
    KeyName {
        name: "6",
        key: KeyCode::Digit6,
    },
    KeyName {
        name: "7",
        key: KeyCode::Digit7,
    },
    KeyName {
        name: "8",
        key: KeyCode::Digit8,
    },
    KeyName {
        name: "9",
        key: KeyCode::Digit9,
    },
    KeyName {
        name: "f1",
        key: KeyCode::F1,
    },
    KeyName {
        name: "f2",
        key: KeyCode::F2,
    },
    KeyName {
        name: "f3",
        key: KeyCode::F3,
    },
    KeyName {
        name: "f4",
        key: KeyCode::F4,
    },
    KeyName {
        name: "f5",
        key: KeyCode::F5,
    },
    KeyName {
        name: "f6",
        key: KeyCode::F6,
    },
    KeyName {
        name: "f7",
        key: KeyCode::F7,
    },
    KeyName {
        name: "f8",
        key: KeyCode::F8,
    },
    KeyName {
        name: "f9",
        key: KeyCode::F9,
    },
    KeyName {
        name: "f10",
        key: KeyCode::F10,
    },
    KeyName {
        name: "f11",
        key: KeyCode::F11,
    },
    KeyName {
        name: "f12",
        key: KeyCode::F12,
    },
    KeyName {
        name: "`",
        key: KeyCode::Backquote,
    },
    KeyName {
        name: "-",
        key: KeyCode::Minus,
    },
    KeyName {
        name: "=",
        key: KeyCode::Equal,
    },
    KeyName {
        name: "[",
        key: KeyCode::BracketLeft,
    },
    KeyName {
        name: "]",
        key: KeyCode::BracketRight,
    },
    KeyName {
        name: "\\",
        key: KeyCode::Backslash,
    },
    KeyName {
        name: ";",
        key: KeyCode::Semicolon,
    },
    KeyName {
        name: "'",
        key: KeyCode::Quote,
    },
    KeyName {
        name: ",",
        key: KeyCode::Comma,
    },
    KeyName {
        name: ".",
        key: KeyCode::Period,
    },
    KeyName {
        name: "/",
        key: KeyCode::Slash,
    },
    KeyName {
        name: "tab",
        key: KeyCode::Tab,
    },
    KeyName {
        name: "escape",
        key: KeyCode::Escape,
    },
    KeyName {
        name: "space",
        key: KeyCode::Space,
    },
    KeyName {
        name: "enter",
        key: KeyCode::Enter,
    },
    KeyName {
        name: "backspace",
        key: KeyCode::Backspace,
    },
    KeyName {
        name: "insert",
        key: KeyCode::Insert,
    },
    KeyName {
        name: "delete",
        key: KeyCode::Delete,
    },
    KeyName {
        name: "home",
        key: KeyCode::Home,
    },
    KeyName {
        name: "end",
        key: KeyCode::End,
    },
    KeyName {
        name: "page-up",
        key: KeyCode::PageUp,
    },
    KeyName {
        name: "page-down",
        key: KeyCode::PageDown,
    },
    KeyName {
        name: "up",
        key: KeyCode::ArrowUp,
    },
    KeyName {
        name: "down",
        key: KeyCode::ArrowDown,
    },
    KeyName {
        name: "left",
        key: KeyCode::ArrowLeft,
    },
    KeyName {
        name: "right",
        key: KeyCode::ArrowRight,
    },
];

impl KeyChord {
    pub fn key_names() -> Vec<&'static str> {
        KEY_NAMES.iter().map(|key| key.name).collect()
    }

    // Modifiers come first: `shift+tab`, `ctrl+alt+f1`.
    pub fn from_config_name(text: &str) -> Option<KeyChord> {
        let text = text.to_lowercase();
        let mut parts: Vec<&str> = text.split(MODIFIER_SEPARATOR).collect();
        let key_name = parts.pop()?;
        let key = KEY_NAMES.iter().find(|key| key.name == key_name)?.key;
        let mut chord = KeyChord::key(key);

        for modifier in parts {
            match modifier {
                SHIFT => chord.shift = true,
                CONTROL => chord.control = true,
                ALT => chord.alt = true,
                _ => return None,
            }
        }

        Some(chord)
    }

    pub fn config_name(&self) -> String {
        let key = KEY_NAMES
            .iter()
            .find(|key| key.key == self.key)
            .map_or("unknown", |key| key.name);
        let mut parts = Vec::new();

        if self.control {
            parts.push(CONTROL);
        }

        if self.alt {
            parts.push(ALT);
        }

        if self.shift {
            parts.push(SHIFT);
        }

        parts.push(key);

        parts.join(&MODIFIER_SEPARATOR.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for name in ["f4", "tab", "shift+tab", "`", "ctrl+alt+delete", "1"] {
            assert_eq!(KeyChord::from_config_name(name).unwrap().config_name(), name);
        }
    }

    #[test]
    fn names_ignore_case_and_reject_unknown_keys() {
        assert_eq!(
            KeyChord::from_config_name("Shift+Tab"),
            Some(KeyChord::shift(KeyCode::Tab))
        );
        assert!(KeyChord::from_config_name("hyper+a").is_none());
        assert!(KeyChord::from_config_name("f99").is_none());
    }
}
