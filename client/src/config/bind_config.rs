use std::collections::HashMap;

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
}
