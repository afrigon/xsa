use std::collections::BTreeMap;

use anyhow::Context;

/// The key/value metadata a texture file carries.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureMetadata {
    entries: BTreeMap<String, Vec<u8>>,
}

impl TextureMetadata {
    pub(super) fn new(entries: BTreeMap<String, Vec<u8>>) -> TextureMetadata {
        TextureMetadata { entries }
    }

    /// The value stored under a key, as the file holds it.
    pub fn value(&self, key: &str) -> Option<&[u8]> {
        self.entries.get(key).map(Vec::as_slice)
    }

    /// The numbers stored under a key, written as text separated by spaces and ending with a NUL.
    pub fn numbers(&self, key: &str) -> anyhow::Result<Vec<f32>> {
        let value = self.value(key).with_context(|| format!("no {key} metadata"))?;
        let text = value
            .strip_suffix(b"\0")
            .and_then(|text| std::str::from_utf8(text).ok())
            .with_context(|| format!("{key} metadata is not text ending with a NUL"))?;

        text.split_ascii_whitespace()
            .map(|number| {
                number
                    .parse()
                    .with_context(|| format!("{key} metadata: {number} is not a number"))
            })
            .collect()
    }
}
