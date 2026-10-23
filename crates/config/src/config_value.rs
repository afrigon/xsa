use std::num::NonZeroUsize;

use anyhow::{Context, bail};
use kdl::KdlValue;

const TRUE_WORDS: [&str; 2] = ["true", "on"];
const FALSE_WORDS: [&str; 2] = ["false", "off"];
const NONE_WORD: &str = "none";

pub trait ConfigValue: Clone + PartialEq {
    fn from_kdl(value: &KdlValue) -> Option<Self>;

    fn to_kdl(&self) -> KdlValue;

    fn parse(text: &str) -> anyhow::Result<Self>;

    // The values toggling steps through, in order; empty when the type does not toggle.
    fn cycle() -> Vec<Self> {
        Vec::new()
    }

    fn completions() -> Vec<String> {
        Self::cycle().iter().map(ConfigValue::text).collect()
    }

    fn text(&self) -> String {
        match self.to_kdl() {
            KdlValue::String(text) => text,
            KdlValue::Bool(true) => TRUE_WORDS[0].to_string(),
            KdlValue::Bool(false) => FALSE_WORDS[0].to_string(),
            other => other.to_string(),
        }
    }

    fn toggled(&self) -> anyhow::Result<Self> {
        let cycle = Self::cycle();

        if cycle.is_empty() {
            bail!("only on/off and choice settings toggle");
        }

        let next = cycle
            .iter()
            .position(|value| value == self)
            .map_or(0, |index| index + 1);

        Ok(cycle[next % cycle.len()].clone())
    }
}

impl ConfigValue for bool {
    fn from_kdl(value: &KdlValue) -> Option<bool> {
        value.as_bool()
    }

    fn to_kdl(&self) -> KdlValue {
        KdlValue::Bool(*self)
    }

    fn parse(text: &str) -> anyhow::Result<bool> {
        match text {
            _ if TRUE_WORDS.contains(&text) => Ok(true),
            _ if FALSE_WORDS.contains(&text) => Ok(false),
            _ => bail!("{text:?} is not true or false"),
        }
    }

    fn cycle() -> Vec<bool> {
        vec![true, false]
    }
}

impl ConfigValue for f32 {
    fn from_kdl(value: &KdlValue) -> Option<f32> {
        value
            .as_float()
            .or_else(|| value.as_integer().map(|integer| integer as f64))
            .filter(|number| number.is_finite())
            .map(|number| number as f32)
    }

    // Through the shortest decimal form, so an f32 0.02 is written as 0.02 rather than 0.019999999552965164.
    fn to_kdl(&self) -> KdlValue {
        KdlValue::Float(self.to_string().parse().unwrap_or(f64::from(*self)))
    }

    fn parse(text: &str) -> anyhow::Result<f32> {
        text.parse()
            .ok()
            .filter(|number: &f32| number.is_finite())
            .with_context(|| format!("{text:?} is not a number"))
    }
}

impl ConfigValue for NonZeroUsize {
    fn from_kdl(value: &KdlValue) -> Option<NonZeroUsize> {
        value
            .as_integer()
            .and_then(|count| usize::try_from(count).ok())
            .and_then(NonZeroUsize::new)
    }

    fn to_kdl(&self) -> KdlValue {
        KdlValue::Integer(self.get() as i128)
    }

    fn parse(text: &str) -> anyhow::Result<NonZeroUsize> {
        text.parse()
            .ok()
            .with_context(|| format!("{text:?} is not a whole number of 1 or more"))
    }
}

// `none`, or a value; `#null` reads as none too.
impl<T: ConfigValue> ConfigValue for Option<T> {
    fn from_kdl(value: &KdlValue) -> Option<Option<T>> {
        match value {
            KdlValue::Null => Some(None),
            KdlValue::String(text) if text == NONE_WORD => Some(None),
            value => T::from_kdl(value).map(Some),
        }
    }

    fn to_kdl(&self) -> KdlValue {
        self.as_ref()
            .map_or_else(|| KdlValue::String(NONE_WORD.to_string()), ConfigValue::to_kdl)
    }

    fn parse(text: &str) -> anyhow::Result<Option<T>> {
        match text {
            NONE_WORD => Ok(None),
            text => T::parse(text).map(Some),
        }
    }

    fn cycle() -> Vec<Option<T>> {
        let values = T::cycle();

        if values.is_empty() {
            return Vec::new();
        }

        [None].into_iter().chain(values.into_iter().map(Some)).collect()
    }

    fn completions() -> Vec<String> {
        [NONE_WORD.to_string()].into_iter().chain(T::completions()).collect()
    }
}
