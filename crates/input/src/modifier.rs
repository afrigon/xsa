use crate::{InputButton, Key};

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

    pub fn left(self) -> Key {
        match self {
            Modifier::Control => Key::ControlLeft,
            Modifier::Alt => Key::AltLeft,
            Modifier::Shift => Key::ShiftLeft,
            Modifier::Super => Key::SuperLeft,
        }
    }

    pub fn right(self) -> Key {
        match self {
            Modifier::Control => Key::ControlRight,
            Modifier::Alt => Key::AltRight,
            Modifier::Shift => Key::ShiftRight,
            Modifier::Super => Key::SuperRight,
        }
    }

    pub fn matches(self, button: InputButton) -> bool {
        button == InputButton::Key(self.left()) || button == InputButton::Key(self.right())
    }
}
