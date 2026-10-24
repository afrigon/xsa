use std::collections::BTreeMap;

use kdl::KdlValue;

use crate::key_path::KeyPath;

// An unset key reads as its default even when a saved value exists.
#[derive(Default)]
pub(crate) struct SessionLayer {
    values: BTreeMap<String, SessionValue>,
}

pub(crate) enum SessionValue {
    Set(KdlValue),
    Unset,
}

impl SessionLayer {
    pub fn get(&self, path: &str) -> Option<&SessionValue> {
        self.values.get(path)
    }

    pub fn set(&mut self, path: &str, value: SessionValue) {
        self.values.insert(path.to_string(), value);
    }

    pub fn remove_within(&mut self, parent: &str) {
        self.values.retain(|path, _| !KeyPath::contains(parent, path));
    }
}
