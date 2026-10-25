use anyhow::bail;
use kdl::KdlValue;

use crate::ConfigValue;

pub trait ConfigChoice: Copy + PartialEq + 'static {
    const ALL: &'static [Self];

    fn config_name(self) -> &'static str;
}

impl<T: ConfigChoice> ConfigValue for T {
    fn from_kdl(value: &KdlValue) -> Option<T> {
        let name = value.as_string()?;

        T::ALL.iter().copied().find(|choice| choice.config_name() == name)
    }

    fn to_kdl(&self) -> KdlValue {
        KdlValue::String(self.config_name().to_string())
    }

    fn parse(text: &str) -> anyhow::Result<T> {
        match T::ALL.iter().copied().find(|choice| choice.config_name() == text) {
            Some(choice) => Ok(choice),
            None => {
                let names: Vec<&str> = T::ALL.iter().map(|choice| choice.config_name()).collect();
                bail!("{text:?} is not one of {}", names.join(", "))
            }
        }
    }

    fn type_name() -> String {
        let names: Vec<&str> = T::ALL.iter().map(|choice| choice.config_name()).collect();

        names.join("|")
    }

    fn cycle() -> Vec<T> {
        T::ALL.to_vec()
    }
}
