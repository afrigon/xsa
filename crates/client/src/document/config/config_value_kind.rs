use anyhow::{Context, bail};
use kdl::KdlValue;

use super::NumberRange;
use crate::config::{ByteSize, KeyChord};

const TRUE_WORDS: [&str; 2] = ["true", "on"];
const FALSE_WORDS: [&str; 2] = ["false", "off"];
const NONE_WORD: &str = "none";

// Building blocks for declaring settings, kept whether or not a setting uses them.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum ConfigValueKind {
    Bool,
    Number { range: NumberRange },
    Choice { names: fn() -> Vec<&'static str> },
    KeyChord,
    // A whole number, 1 or more.
    Count,
    ByteSize,
    // `value`, or none.
    Optional { value: &'static ConfigValueKind },
}

impl ConfigValueKind {
    // A building block for declaring settings, kept whether or not a setting uses it.
    #[allow(dead_code)]
    pub const NUMBER: ConfigValueKind = ConfigValueKind::Number {
        range: NumberRange::ANY,
    };

    // Through the shortest decimal form, so an f32 0.02 is written as 0.02 rather than 0.019999999552965164.
    pub fn number(value: f32) -> KdlValue {
        KdlValue::Float(value.to_string().parse().unwrap_or(f64::from(value)))
    }

    pub fn parse(self, text: &str) -> anyhow::Result<KdlValue> {
        match self {
            ConfigValueKind::Bool if TRUE_WORDS.contains(&text) => Ok(KdlValue::Bool(true)),
            ConfigValueKind::Bool if FALSE_WORDS.contains(&text) => Ok(KdlValue::Bool(false)),
            ConfigValueKind::Bool => bail!("{text:?} is not true or false"),
            ConfigValueKind::Number { range } => {
                let number: f32 = text
                    .parse()
                    .ok()
                    .filter(|number: &f32| number.is_finite())
                    .context(format!("{text:?} is not a number"))?;

                if !range.contains(number) {
                    bail!("{number} is out of range: expected {range}");
                }

                Ok(ConfigValueKind::number(number))
            }
            ConfigValueKind::Choice { names } if names().contains(&text) => Ok(KdlValue::String(text.to_string())),
            ConfigValueKind::Choice { names } => bail!("{text:?} is not one of {}", names().join(", ")),
            ConfigValueKind::KeyChord => {
                let chord = KeyChord::from_config_name(text)
                    .with_context(|| format!("{text:?} is not a key like f4, tab, shift+tab or `"))?;

                Ok(KdlValue::String(chord.config_name()))
            }
            ConfigValueKind::Count => {
                let count = text
                    .parse::<u32>()
                    .ok()
                    .filter(|count| *count > 0)
                    .with_context(|| format!("{text:?} is not a whole number of 1 or more"))?;

                Ok(KdlValue::Integer(count.into()))
            }
            ConfigValueKind::ByteSize => {
                let size = ByteSize::from_config_name(text)
                    .with_context(|| format!("{text:?} is not a size like 512MiB, 2GB or 1024B"))?;

                Ok(KdlValue::String(size.config_name()))
            }
            ConfigValueKind::Optional { .. } if text == NONE_WORD => Ok(KdlValue::Null),
            ConfigValueKind::Optional { value } => value.parse(text),
        }
    }

    pub fn accepts_number(self, number: f32) -> bool {
        match self {
            ConfigValueKind::Number { range } => range.contains(number),
            ConfigValueKind::Optional { value } => value.accepts_number(number),
            _ => false,
        }
    }

    pub fn values(self) -> Vec<&'static str> {
        match self {
            ConfigValueKind::Bool => vec![TRUE_WORDS[0], FALSE_WORDS[0]],
            ConfigValueKind::Number { .. } => Vec::new(),
            ConfigValueKind::Choice { names } => names(),
            ConfigValueKind::KeyChord => KeyChord::key_names(),
            ConfigValueKind::Count | ConfigValueKind::ByteSize => Vec::new(),
            ConfigValueKind::Optional { value } => [NONE_WORD].into_iter().chain(value.values()).collect(),
        }
    }

    pub fn same(self, first: &KdlValue, second: &KdlValue) -> bool {
        match self {
            ConfigValueKind::Number { .. } => ConfigValueKind::as_f32(first) == ConfigValueKind::as_f32(second),
            ConfigValueKind::KeyChord => {
                ConfigValueKind::as_chord(first).is_some()
                    && ConfigValueKind::as_chord(first) == ConfigValueKind::as_chord(second)
            }
            ConfigValueKind::Optional { value } if !first.is_null() && !second.is_null() => value.same(first, second),
            ConfigValueKind::Bool
            | ConfigValueKind::Choice { .. }
            | ConfigValueKind::Count
            | ConfigValueKind::ByteSize
            | ConfigValueKind::Optional { .. } => first == second,
        }
    }

    pub fn toggled(self, current: &KdlValue) -> anyhow::Result<KdlValue> {
        match self {
            ConfigValueKind::Bool => Ok(KdlValue::Bool(!current.as_bool().unwrap_or(false))),
            ConfigValueKind::Number { .. }
            | ConfigValueKind::KeyChord
            | ConfigValueKind::Count
            | ConfigValueKind::ByteSize
            | ConfigValueKind::Optional { .. } => {
                bail!("only on/off and choice settings toggle")
            }
            ConfigValueKind::Choice { names } => {
                let names = names();
                let index = names
                    .iter()
                    .position(|name| Some(*name) == current.as_string())
                    .unwrap_or(0);

                Ok(KdlValue::String(names[(index + 1) % names.len()].to_string()))
            }
        }
    }

    pub fn text(value: &KdlValue) -> String {
        match value {
            KdlValue::String(text) => text.clone(),
            KdlValue::Bool(true) => TRUE_WORDS[0].to_string(),
            KdlValue::Bool(false) => FALSE_WORDS[0].to_string(),
            KdlValue::Null => NONE_WORD.to_string(),
            other => other.to_string(),
        }
    }

    fn as_chord(value: &KdlValue) -> Option<KeyChord> {
        value.as_string().and_then(KeyChord::from_config_name)
    }

    fn as_f32(value: &KdlValue) -> Option<f32> {
        value
            .as_float()
            .or_else(|| value.as_integer().map(|integer| integer as f64))
            .map(|number| number as f32)
    }
}
