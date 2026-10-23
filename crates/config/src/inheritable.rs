use kdl::KdlValue;

use crate::ConfigValue;

const INHERIT_WORD: &str = "inherit";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Inheritable<T> {
    Inherit,
    Value(T),
}

impl<T: Clone> Inheritable<T> {
    pub fn or(&self, inherited: &T) -> T {
        match self {
            Inheritable::Inherit => inherited.clone(),
            Inheritable::Value(value) => value.clone(),
        }
    }
}

impl<T: ConfigValue> ConfigValue for Inheritable<T> {
    fn from_kdl(value: &KdlValue) -> Option<Inheritable<T>> {
        match value.as_string() {
            Some(INHERIT_WORD) => Some(Inheritable::Inherit),
            _ => T::from_kdl(value).map(Inheritable::Value),
        }
    }

    fn to_kdl(&self) -> KdlValue {
        match self {
            Inheritable::Inherit => KdlValue::String(INHERIT_WORD.to_string()),
            Inheritable::Value(value) => value.to_kdl(),
        }
    }

    fn parse(text: &str) -> anyhow::Result<Inheritable<T>> {
        match text {
            INHERIT_WORD => Ok(Inheritable::Inherit),
            text => T::parse(text).map(Inheritable::Value),
        }
    }

    fn completions() -> Vec<String> {
        [INHERIT_WORD.to_string()].into_iter().chain(T::completions()).collect()
    }
}
