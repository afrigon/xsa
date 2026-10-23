use std::collections::HashMap;

use xsa_config::{ConfigSection, ConfigVisitor};

use super::{BindAction, KeyChord};
use crate::input::Input;

#[derive(Clone, Debug, PartialEq)]
pub struct BindConfig {
    pub keys: HashMap<BindAction, KeyChord>,
}

impl BindConfig {
    pub fn triggered(&self, input: &Input) -> Vec<BindAction> {
        BindAction::ALL
            .into_iter()
            .filter(|action| self.keys.get(action).is_some_and(|chord| chord.was_pressed(input)))
            .collect()
    }

    pub fn shared_chords(&self) -> Vec<KeyChord> {
        let mut shared: Vec<KeyChord> = Vec::new();

        for chord in self.keys.values() {
            let count = self.keys.values().filter(|other| *other == chord).count();

            if count > 1 && !shared.contains(chord) {
                shared.push(*chord);
            }
        }

        shared
    }
}

impl ConfigSection for BindConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        for action in BindAction::ALL {
            if let Some(chord) = self.keys.get_mut(&action) {
                visitor.group(action.category(), |visitor| visitor.value(action.name(), chord));
            }
        }
    }
}
