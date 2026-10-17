use kdl::KdlValue;

use super::{ConfigChoice, ConfigKey, ConfigLayer};
use crate::config::{ByteSize, KeyChord};

// The session layer takes precedence; only persisted keys are read from the file.
pub struct ConfigValues<'a> {
    pub session: &'a ConfigLayer,
    pub file: &'a ConfigLayer,
}

impl ConfigValues<'_> {
    pub fn bool(&self, key: ConfigKey) -> Option<bool> {
        let value = self.value(key)?;
        let parsed = value.as_bool();
        ConfigValues::warn_if_invalid(key, value, parsed.is_some());

        parsed
    }

    pub fn number(&self, key: ConfigKey) -> Option<f32> {
        let value = self.value(key)?;
        let parsed = value
            .as_float()
            .or_else(|| value.as_integer().map(|integer| integer as f64))
            .filter(|number| number.is_finite())
            .map(|number| number as f32)
            .filter(|number| key.kind.accepts_number(*number));
        ConfigValues::warn_if_invalid(key, value, parsed.is_some());

        parsed
    }

    pub fn choice<T: ConfigChoice>(&self, key: ConfigKey) -> Option<T> {
        let value = self.value(key)?;
        let parsed = value.as_string().and_then(T::from_config_name);
        ConfigValues::warn_if_invalid(key, value, parsed.is_some());

        parsed
    }

    pub fn key_chord(&self, key: ConfigKey) -> Option<KeyChord> {
        let value = self.value(key)?;
        let parsed = value.as_string().and_then(KeyChord::from_config_name);
        ConfigValues::warn_if_invalid(key, value, parsed.is_some());

        parsed
    }

    // The outer None when unset, the inner one for `none`.
    pub fn optional_count(&self, key: ConfigKey) -> Option<Option<u32>> {
        self.optional(key, |value| {
            value
                .as_integer()
                .and_then(|count| u32::try_from(count).ok())
                .filter(|count| *count > 0)
        })
    }

    // The outer None when unset, the inner one for `none`.
    pub fn optional_byte_size(&self, key: ConfigKey) -> Option<Option<ByteSize>> {
        self.optional(key, |value| value.as_string().and_then(ByteSize::from_config_name))
    }

    fn optional<T>(&self, key: ConfigKey, parse: impl Fn(&KdlValue) -> Option<T>) -> Option<Option<T>> {
        let value = self.value(key)?;

        if value.is_null() {
            return Some(None);
        }

        let parsed = parse(value);
        ConfigValues::warn_if_invalid(key, value, parsed.is_some());

        parsed.map(Some)
    }

    fn value(&self, key: ConfigKey) -> Option<&KdlValue> {
        let file = key.persisted.then_some(self.file);

        [Some(self.session), file]
            .into_iter()
            .flatten()
            .find_map(|layer| layer.get(key.path))
    }

    fn warn_if_invalid(key: ConfigKey, value: &KdlValue, valid: bool) {
        if !valid {
            tracing::warn!("config {}: {value} is not valid, using the default", key.path);
        }
    }
}
