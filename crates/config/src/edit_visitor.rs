use anyhow::Context;
use kdl::KdlValue;

use crate::key_path::KeyPath;
use crate::{ConfigValue, ConfigVisitor};

pub(crate) enum Edit<'a> {
    Parse(&'a str),
    Toggle,
    Assign(&'a KdlValue),
}

pub(crate) struct EditVisitor<'a> {
    path: KeyPath,
    target: &'a str,
    edit: Edit<'a>,
    result: Option<anyhow::Result<KdlValue>>,
}

impl<'a> EditVisitor<'a> {
    pub fn new(target: &'a str, edit: Edit<'a>) -> EditVisitor<'a> {
        EditVisitor {
            path: KeyPath::default(),
            target,
            edit,
            result: None,
        }
    }

    pub fn into_result(self) -> anyhow::Result<KdlValue> {
        self.result
            .with_context(|| format!("unknown config key {}", self.target))?
    }
}

impl ConfigVisitor for EditVisitor<'_> {
    fn enter(&mut self, name: &str) {
        self.path.enter(name);
    }

    fn leave(&mut self) {
        self.path.leave();
    }

    fn checked<T: ConfigValue>(&mut self, name: &str, value: &mut T, check: &dyn Fn(&T) -> anyhow::Result<()>) {
        if self.path.child(name) != self.target {
            return;
        }

        let edited = match self.edit {
            Edit::Parse(text) => T::parse(text),
            Edit::Toggle => value.toggled(),
            Edit::Assign(raw) => T::from_kdl(raw).with_context(|| format!("{raw} is not valid")),
        };
        self.result = Some(edited.and_then(|edited| check(&edited).map(|()| edited.to_kdl())));
    }
}
