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

    pub fn keys(self) -> [Key; 2] {
        match self {
            Modifier::Control => [Key::ControlLeft, Key::ControlRight],
            Modifier::Alt => [Key::AltLeft, Key::AltRight],
            Modifier::Shift => [Key::ShiftLeft, Key::ShiftRight],
            Modifier::Super => [Key::SuperLeft, Key::SuperRight],
        }
    }

    pub fn matches(self, button: InputButton) -> bool {
        self.keys().into_iter().any(|key| button == InputButton::Key(key))
    }
}
