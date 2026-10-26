use xsa_config::{ConfigSection, ConfigVisitor};
use xsa_input::InputChord;

use super::BindCommand;

// A key chord and the commands it runs: `down` as it is pressed and `up`, if any, as its last key is released.
#[derive(Clone, Debug, PartialEq)]
pub struct Bind {
    pub key: InputChord,
    pub down: BindCommand,
    pub up: Option<BindCommand>,
}

impl ConfigSection for Bind {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("key", &mut self.key);
        visitor.value("down", &mut self.down);
        visitor.value("up", &mut self.up);
    }
}
