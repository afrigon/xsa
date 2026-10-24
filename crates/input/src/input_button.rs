use crate::{Key, MouseButton};

const MOUSE_PREFIX: &str = "mouse-";
const MOUSE_LEFT: &str = "left";
const MOUSE_RIGHT: &str = "right";
const MOUSE_MIDDLE: &str = "middle";
const MOUSE_BACK: &str = "back";
const MOUSE_FORWARD: &str = "forward";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputButton {
    Key(Key),
    Mouse(MouseButton),
}

impl InputButton {
    pub fn names() -> Vec<String> {
        let mouse = [MOUSE_LEFT, MOUSE_RIGHT, MOUSE_MIDDLE, MOUSE_BACK, MOUSE_FORWARD]
            .into_iter()
            .map(|name| format!("{MOUSE_PREFIX}{name}"));

        Key::ALL.iter().map(|key| key.name().to_string()).chain(mouse).collect()
    }

    pub fn from_name(name: &str) -> Option<InputButton> {
        if let Some(key) = Key::from_name(name) {
            return Some(InputButton::Key(key));
        }

        let button = match name.strip_prefix(MOUSE_PREFIX)? {
            MOUSE_LEFT => MouseButton::Left,
            MOUSE_RIGHT => MouseButton::Right,
            MOUSE_MIDDLE => MouseButton::Middle,
            MOUSE_BACK => MouseButton::Back,
            MOUSE_FORWARD => MouseButton::Forward,
            number => MouseButton::Other {
                number: number.parse().ok()?,
            },
        };

        Some(InputButton::Mouse(button))
    }

    pub fn name(&self) -> String {
        match self {
            InputButton::Key(key) => key.name().to_string(),
            InputButton::Mouse(button) => {
                let name = match button {
                    MouseButton::Left => MOUSE_LEFT.to_string(),
                    MouseButton::Right => MOUSE_RIGHT.to_string(),
                    MouseButton::Middle => MOUSE_MIDDLE.to_string(),
                    MouseButton::Back => MOUSE_BACK.to_string(),
                    MouseButton::Forward => MOUSE_FORWARD.to_string(),
                    MouseButton::Other { number } => number.to_string(),
                };

                format!("{MOUSE_PREFIX}{name}")
            }
        }
    }
}
