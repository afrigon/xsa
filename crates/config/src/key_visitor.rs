use crate::key_path::KeyPath;
use crate::{ConfigKey, ConfigValue, ConfigVisitor};

#[derive(Default)]
pub(crate) struct KeyVisitor {
    path: KeyPath,
    keys: Vec<ConfigKey>,
}

impl KeyVisitor {
    pub fn into_keys(self) -> Vec<ConfigKey> {
        self.keys
    }
}

impl ConfigVisitor for KeyVisitor {
    fn enter(&mut self, name: &str) {
        self.path.enter(name);
    }

    fn leave(&mut self) {
        self.path.leave();
    }

    fn checked<T: ConfigValue>(&mut self, name: &str, value: &mut T, _check: &dyn Fn(&T) -> anyhow::Result<()>) {
        self.keys.push(ConfigKey {
            path: self.path.child(name),
            value: value.to_kdl(),
            text: value.text(),
            completions: T::completions(),
        });
    }
}
