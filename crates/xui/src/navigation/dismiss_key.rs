use crate::{Action, EnvironmentKey};

// Removes the view controller whose view reads it from its navigation stack, like SwiftUI's `dismiss`.
pub struct DismissKey;

impl EnvironmentKey for DismissKey {
    type Value = Action;

    fn default_value() -> Action {
        Action::none()
    }
}
