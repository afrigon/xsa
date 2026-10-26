use xsa_config::{ConfigSection, ConfigVisitor};
use xsa_input::InputChord;

use super::{Bind, NamedBind};
use crate::input::Input;

#[derive(Clone, Debug, PartialEq)]
pub struct BindConfig {
    pub binds: Vec<NamedBind>,
}

impl BindConfig {
    // The binds this frame's presses fire. With shift held, a press of tab fires shift+tab and not tab.
    pub fn pressed(&self, input: &Input) -> Vec<&Bind> {
        let mut pressed = Vec::new();

        for press in input.presses() {
            let matching: Vec<&Bind> = self
                .binds
                .iter()
                .map(|named| &named.bind)
                .filter(|bind| bind.key.matches(press))
                .collect();
            let most_specific = matching.iter().map(|bind| bind.key.specificity()).max();
            pressed.extend(
                matching
                    .into_iter()
                    .filter(|bind| Some(bind.key.specificity()) == most_specific),
            );
        }

        pressed
    }

    pub fn shared_chords(&self) -> Vec<InputChord> {
        let mut shared: Vec<InputChord> = Vec::new();

        for named in &self.binds {
            let chord = &named.bind.key;
            let count = self.binds.iter().filter(|other| other.bind.key == *chord).count();

            if count > 1 && !shared.contains(chord) {
                shared.push(chord.clone());
            }
        }

        shared
    }
}

impl ConfigSection for BindConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        for named in &mut self.binds {
            visitor.group(named.category, |visitor| visitor.section(named.name, &mut named.bind));
        }
    }
}
