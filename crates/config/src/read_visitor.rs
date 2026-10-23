use crate::config_layer::ConfigLayer;
use crate::key_path::KeyPath;
use crate::session_layer::{SessionLayer, SessionValue};
use crate::{ConfigValue, ConfigVisitor};

// The section starts out holding its defaults; an invalid value keeps the default.
pub(crate) struct ReadVisitor<'a> {
    path: KeyPath,
    saved: &'a ConfigLayer,
    session: &'a SessionLayer,
}

impl<'a> ReadVisitor<'a> {
    pub fn new(saved: &'a ConfigLayer, session: &'a SessionLayer) -> ReadVisitor<'a> {
        ReadVisitor {
            path: KeyPath::default(),
            saved,
            session,
        }
    }
}

impl ConfigVisitor for ReadVisitor<'_> {
    fn enter(&mut self, name: &str) {
        self.path.enter(name);
    }

    fn leave(&mut self) {
        self.path.leave();
    }

    fn checked<T: ConfigValue>(&mut self, name: &str, value: &mut T, check: &dyn Fn(&T) -> anyhow::Result<()>) {
        let path = self.path.child(name);
        let raw = match self.session.get(&path) {
            Some(SessionValue::Unset) => return,
            Some(SessionValue::Set(raw)) => raw,
            None => match self.saved.get(&path) {
                Some(raw) => raw,
                None => return,
            },
        };

        match T::from_kdl(raw).filter(|parsed| check(parsed).is_ok()) {
            Some(parsed) => *value = parsed,
            None => tracing::warn!("config {path}: {raw} is not valid, using the default"),
        }
    }
}
