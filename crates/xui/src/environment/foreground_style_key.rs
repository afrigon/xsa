use super::EnvironmentKey;
use crate::LinearColor;

pub struct ForegroundStyleKey;

impl EnvironmentKey for ForegroundStyleKey {
    type Value = LinearColor;

    fn default_value() -> LinearColor {
        LinearColor::WHITE
    }
}
