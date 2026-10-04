use std::rc::Rc;

use super::{AnyButtonStyle, PlainButtonStyle};
use crate::EnvironmentKey;

pub struct ButtonStyleKey;

impl EnvironmentKey for ButtonStyleKey {
    type Value = Rc<dyn AnyButtonStyle>;

    fn default_value() -> Rc<dyn AnyButtonStyle> {
        Rc::new(PlainButtonStyle)
    }
}
