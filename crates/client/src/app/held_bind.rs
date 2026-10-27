use xsa_input::InputChord;

use crate::config::BindCommand;

// A bind held down, which runs its `up` command once its chord's last key is released.
pub(super) struct HeldBind {
    pub key: InputChord,
    pub up: BindCommand,
}
