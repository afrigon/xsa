use winit::keyboard::KeyCode;

use crate::input::Input;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub key: KeyCode,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
}

impl KeyChord {
    pub const fn key(key: KeyCode) -> KeyChord {
        KeyChord {
            key,
            shift: false,
            control: false,
            alt: false,
        }
    }

    pub const fn shift(key: KeyCode) -> KeyChord {
        KeyChord {
            shift: true,
            ..KeyChord::key(key)
        }
    }

    // The modifiers must match exactly, so Tab and Shift+Tab are different chords.
    pub fn was_pressed(&self, input: &Input) -> bool {
        let shift = input.is_held(KeyCode::ShiftLeft) || input.is_held(KeyCode::ShiftRight);
        let control = input.is_held(KeyCode::ControlLeft) || input.is_held(KeyCode::ControlRight);
        let alt = input.is_held(KeyCode::AltLeft) || input.is_held(KeyCode::AltRight);

        input.was_pressed(self.key) && self.shift == shift && self.control == control && self.alt == alt
    }
}
