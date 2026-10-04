use xui::EnvironmentKey;

use super::Actions;

pub struct ActionsKey;

impl EnvironmentKey for ActionsKey {
    type Value = Option<Actions>;

    fn default_value() -> Option<Actions> {
        None
    }
}
