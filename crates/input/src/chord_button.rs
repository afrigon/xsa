use crate::{InputButton, Modifier};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChordButton {
    Modifier(Modifier),
    Button(InputButton),
}

impl ChordButton {
    pub fn from_name(name: &str) -> Option<ChordButton> {
        Modifier::from_name(name)
            .map(ChordButton::Modifier)
            .or_else(|| InputButton::from_name(name).map(ChordButton::Button))
    }

    pub fn name(&self) -> String {
        match self {
            ChordButton::Modifier(modifier) => modifier.name().to_string(),
            ChordButton::Button(button) => button.name(),
        }
    }

    pub fn matches(&self, button: InputButton) -> bool {
        match self {
            ChordButton::Modifier(modifier) => modifier.matches(button),
            ChordButton::Button(own) => *own == button,
        }
    }
}
