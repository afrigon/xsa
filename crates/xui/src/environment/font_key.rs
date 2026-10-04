use super::EnvironmentKey;
use crate::Font;

pub struct FontKey;

impl EnvironmentKey for FontKey {
    type Value = Option<Font>;

    fn default_value() -> Option<Font> {
        None
    }
}
