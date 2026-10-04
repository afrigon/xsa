mod color_scheme_key;
mod environment_key;
mod font_key;
mod foreground_style_key;
mod opacity_key;
mod scale_factor_key;

pub use color_scheme_key::ColorSchemeKey;
pub use environment_key::EnvironmentKey;
pub use font_key::FontKey;
pub use foreground_style_key::ForegroundStyleKey;
pub use opacity_key::OpacityKey;
pub use scale_factor_key::ScaleFactorKey;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct Environment {
    values: HashMap<TypeId, Rc<dyn Any>>,
}

impl Environment {
    pub fn get<K: EnvironmentKey>(&self) -> K::Value {
        self.values
            .get(&TypeId::of::<K>())
            .and_then(|value| value.downcast_ref::<K::Value>())
            .cloned()
            .unwrap_or_else(K::default_value)
    }

    pub fn with<K: EnvironmentKey>(&self, value: K::Value) -> Environment {
        let mut environment = self.clone();
        environment.values.insert(TypeId::of::<K>(), Rc::new(value));
        environment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorScheme;

    #[test]
    fn reads_defaults_until_a_value_is_set() {
        let environment = Environment::default();
        assert_eq!(environment.get::<ColorSchemeKey>(), ColorScheme::Light);

        let dark = environment.with::<ColorSchemeKey>(ColorScheme::Dark);
        assert_eq!(dark.get::<ColorSchemeKey>(), ColorScheme::Dark);
        assert_eq!(environment.get::<ColorSchemeKey>(), ColorScheme::Light);
    }
}
