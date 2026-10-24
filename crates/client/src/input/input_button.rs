use winit::event::MouseButton;
use winit::keyboard::KeyCode;

const MOUSE_PREFIX: &str = "mouse-";
const MOUSE_LEFT: &str = "left";
const MOUSE_RIGHT: &str = "right";
const MOUSE_MIDDLE: &str = "middle";
const MOUSE_BACK: &str = "back";
const MOUSE_FORWARD: &str = "forward";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputButton {
    Key(KeyCode),
    Mouse(MouseButton),
}

struct KeyName {
    name: &'static str,
    key: KeyCode,
}

const KEY_NAMES: [KeyName; 82] = [
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
    KeyName {
        name: "left-shift",
        key: KeyCode::ShiftLeft,
    },
    KeyName {
        name: "right-shift",
        key: KeyCode::ShiftRight,
    },
    KeyName {
        name: "left-ctrl",
        key: KeyCode::ControlLeft,
    },
    KeyName {
        name: "right-ctrl",
        key: KeyCode::ControlRight,
    },
    KeyName {
        name: "left-alt",
        key: KeyCode::AltLeft,
    },
    KeyName {
        name: "right-alt",
        key: KeyCode::AltRight,
    },
    KeyName {
        name: "left-super",
        key: KeyCode::SuperLeft,
    },
    KeyName {
        name: "right-super",
        key: KeyCode::SuperRight,
    },
];

impl InputButton {
    pub fn names() -> Vec<String> {
        let mouse = [MOUSE_LEFT, MOUSE_RIGHT, MOUSE_MIDDLE, MOUSE_BACK, MOUSE_FORWARD]
            .into_iter()
            .map(|name| format!("{MOUSE_PREFIX}{name}"));

        KEY_NAMES.iter().map(|key| key.name.to_string()).chain(mouse).collect()
    }

    pub fn from_name(name: &str) -> Option<InputButton> {
        if let Some(key) = KEY_NAMES.iter().find(|key| key.name == name) {
            return Some(InputButton::Key(key.key));
        }

        let button = match name.strip_prefix(MOUSE_PREFIX)? {
            MOUSE_LEFT => MouseButton::Left,
            MOUSE_RIGHT => MouseButton::Right,
            MOUSE_MIDDLE => MouseButton::Middle,
            MOUSE_BACK => MouseButton::Back,
            MOUSE_FORWARD => MouseButton::Forward,
            number => MouseButton::Other(number.parse().ok()?),
        };

        Some(InputButton::Mouse(button))
    }

    pub fn name(&self) -> String {
        match self {
            InputButton::Key(key) => KEY_NAMES
                .iter()
                .find(|name| name.key == *key)
                .map_or_else(|| format!("{key:?}"), |name| name.name.to_string()),
            InputButton::Mouse(button) => {
                let name = match button {
                    MouseButton::Left => MOUSE_LEFT.to_string(),
                    MouseButton::Right => MOUSE_RIGHT.to_string(),
                    MouseButton::Middle => MOUSE_MIDDLE.to_string(),
                    MouseButton::Back => MOUSE_BACK.to_string(),
                    MouseButton::Forward => MOUSE_FORWARD.to_string(),
                    MouseButton::Other(number) => number.to_string(),
                };

                format!("{MOUSE_PREFIX}{name}")
            }
        }
    }
}
