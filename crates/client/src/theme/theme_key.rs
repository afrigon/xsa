use std::rc::Rc;

use xui::EnvironmentKey;

use super::Theme;

pub struct ThemeKey;

impl EnvironmentKey for ThemeKey {
    type Value = Option<Rc<Theme>>;

    fn default_value() -> Option<Rc<Theme>> {
        None
    }
}
