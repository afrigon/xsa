use super::EnvironmentKey;
use crate::ColorScheme;

pub struct ColorSchemeKey;

impl EnvironmentKey for ColorSchemeKey {
    type Value = ColorScheme;

    fn default_value() -> ColorScheme {
        ColorScheme::Light
    }
}
