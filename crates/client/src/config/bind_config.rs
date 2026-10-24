use std::collections::HashMap;

use xsa_config::{ConfigSection, ConfigVisitor};

use super::BindAction;
use xsa_input::InputChord;

use crate::input::Input;

#[derive(Clone, Debug, PartialEq)]
pub struct BindConfig {
    pub keys: HashMap<BindAction, InputChord>,
}

impl BindConfig {
    // With shift held, a press of tab fires shift+tab and not tab.
    pub fn triggered(&self, input: &Input) -> Vec<BindAction> {
        let mut triggered = Vec::new();

        for press in input.presses() {
            let matching: Vec<BindAction> = BindAction::ALL
                .into_iter()
                .filter(|action| self.keys.get(action).is_some_and(|chord| chord.matches(press)))
                .collect();
            let specificity = |action: &BindAction| self.keys.get(action).map(InputChord::specificity);
            let most_specific = matching.iter().filter_map(specificity).max();
            triggered.extend(
                matching
                    .into_iter()
                    .filter(|action| specificity(action) == most_specific),
            );
        }

        triggered
    }

    pub fn shared_chords(&self) -> Vec<InputChord> {
        let mut shared: Vec<InputChord> = Vec::new();

        for chord in self.keys.values() {
            let count = self.keys.values().filter(|other| *other == chord).count();

            if count > 1 && !shared.contains(chord) {
                shared.push(chord.clone());
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
