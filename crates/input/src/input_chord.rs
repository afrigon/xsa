use std::str::FromStr;

use anyhow::Context;
use xsa_config::{ConfigValue, KdlValue};

use crate::{ButtonPress, ChordButton, InputButton, Key, Modifier};

const SEPARATOR: char = '+';

// The trigger is pressed last; the held buttons go down before it in any order, and other held buttons don't matter.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputChord {
    held: Vec<ChordButton>,
    trigger: ChordButton,
}

impl InputChord {
    pub fn new(mut held: Vec<ChordButton>, trigger: ChordButton) -> InputChord {
        held.sort();
        held.dedup();

        InputChord { held, trigger }
    }

    pub fn key(key: Key) -> InputChord {
        InputChord::new(Vec::new(), ChordButton::Button(InputButton::Key(key)))
    }

    pub fn with_modifier(modifier: Modifier, key: Key) -> InputChord {
        InputChord::new(
            vec![ChordButton::Modifier(modifier)],
            ChordButton::Button(InputButton::Key(key)),
        )
    }

    pub fn from_name(text: &str) -> Option<InputChord> {
        let text = text.to_lowercase();
        let mut buttons = text
            .split(SEPARATOR)
            .map(ChordButton::from_name)
            .collect::<Option<Vec<ChordButton>>>()?;
        let trigger = buttons.pop()?;

        Some(InputChord::new(buttons, trigger))
    }

    pub fn name(&self) -> String {
        self.buttons()
            .map(|button| button.name())
            .collect::<Vec<String>>()
            .join(&SEPARATOR.to_string())
    }

    pub fn buttons(&self) -> impl Iterator<Item = &ChordButton> {
        self.held.iter().chain([&self.trigger])
    }

    // How many buttons the chord names: among the chords a press matches, only the longest fire.
    pub fn specificity(&self) -> usize {
        self.held.len() + 1
    }

    pub fn matches(&self, press: &ButtonPress) -> bool {
        self.trigger.matches(press.button)
            && self
                .held
                .iter()
                .all(|button| press.held.iter().any(|held| button.matches(*held)))
    }
}

impl FromStr for InputChord {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> anyhow::Result<InputChord> {
        InputChord::from_name(text)
            .with_context(|| format!("{text:?} is not a chord like f4, shift+tab, x+w or ctrl+mouse-left"))
    }
}

impl ConfigValue for InputChord {
    fn from_kdl(value: &KdlValue) -> Option<InputChord> {
        value.as_string().and_then(InputChord::from_name)
    }

    fn to_kdl(&self) -> KdlValue {
        KdlValue::String(self.name())
    }

    fn parse(text: &str) -> anyhow::Result<InputChord> {
        text.parse()
    }

    fn type_name() -> String {
        "input-chord".to_string()
    }

    fn completions() -> Vec<String> {
        Modifier::ALL
            .into_iter()
            .map(|modifier| modifier.name().to_string())
            .chain(InputButton::names())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MouseButton;

    fn chord(name: &str) -> InputChord {
        InputChord::from_name(name).unwrap()
    }

    fn press(button: InputButton, held: &[Key]) -> ButtonPress {
        ButtonPress {
            button,
            held: held.iter().map(|key| InputButton::Key(*key)).collect(),
        }
    }

    #[test]
    fn names_round_trip() {
        for name in [
            "f4",
            "tab",
            "shift+tab",
            "`",
            "ctrl+alt+delete",
            "1",
            "x+w",
            "super+shift+p",
            "left-shift",
            "ctrl+mouse-left",
            "mouse-12",
        ] {
            assert_eq!(chord(name).name(), name.replace("super+shift", "shift+super"));
        }
    }

    #[test]
    fn held_buttons_print_in_one_order() {
        assert_eq!(chord("shift+ctrl+s"), chord("ctrl+shift+s"));
        assert_eq!(chord("shift+ctrl+s").name(), "ctrl+shift+s");
    }

    #[test]
    fn names_ignore_case_and_reject_unknown_buttons() {
        assert_eq!(chord("Shift+Tab"), InputChord::with_modifier(Modifier::Shift, Key::Tab));
        assert!(InputChord::from_name("hyper+a").is_none());
        assert!(InputChord::from_name("f99").is_none());
        assert!(InputChord::from_name("mouse-wheel").is_none());
    }

    #[test]
    fn side_less_modifiers_match_either_key() {
        let shift_tab = chord("shift+tab");
        let tab = InputButton::Key(Key::Tab);

        assert!(shift_tab.matches(&press(tab, &[Key::ShiftLeft])));
        assert!(shift_tab.matches(&press(tab, &[Key::ShiftRight])));
        assert!(!chord("left-shift+tab").matches(&press(tab, &[Key::ShiftRight])));
    }

    #[test]
    fn extra_held_buttons_do_not_prevent_a_match() {
        let tab = InputButton::Key(Key::Tab);

        assert!(chord("tab").matches(&press(tab, &[Key::W])));
        assert!(chord("shift+tab").matches(&press(tab, &[Key::W, Key::ShiftLeft])));
    }

    #[test]
    fn the_trigger_comes_last() {
        let x_w = chord("x+w");

        assert!(x_w.matches(&press(InputButton::Key(Key::W), &[Key::X])));
        assert!(!x_w.matches(&press(InputButton::Key(Key::X), &[Key::W])));
        assert!(!chord("shift+tab").matches(&press(InputButton::Key(Key::Tab), &[])));
    }

    #[test]
    fn mouse_buttons_are_chord_members() {
        let click = InputButton::Mouse(MouseButton::Left);

        assert!(chord("ctrl+mouse-left").matches(&press(click, &[Key::ControlRight])));
        assert!(chord("mouse-back").matches(&press(InputButton::Mouse(MouseButton::Back), &[])));
    }
}
