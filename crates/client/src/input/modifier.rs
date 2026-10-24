use winit::keyboard::KeyCode;

use crate::input::InputButton;

// Either key of a pair, so `shift` matches both shift keys. The order is the order chords print their held buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Modifier {
    Control,
    Alt,
    Shift,
    Super,
}

impl Modifier {
    pub const ALL: [Modifier; 4] = [Modifier::Control, Modifier::Alt, Modifier::Shift, Modifier::Super];

    pub fn from_name(name: &str) -> Option<Modifier> {
        Modifier::ALL.into_iter().find(|modifier| modifier.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            Modifier::Control => "ctrl",
            Modifier::Alt => "alt",
            Modifier::Shift => "shift",
            Modifier::Super => "super",
        }
    }

    pub fn keys(self) -> [KeyCode; 2] {
        match self {
            Modifier::Control => [KeyCode::ControlLeft, KeyCode::ControlRight],
            Modifier::Alt => [KeyCode::AltLeft, KeyCode::AltRight],
            Modifier::Shift => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            Modifier::Super => [KeyCode::SuperLeft, KeyCode::SuperRight],
        }
    }

    pub fn matches(self, button: InputButton) -> bool {
        self.keys().into_iter().any(|key| button == InputButton::Key(key))
    }
}
