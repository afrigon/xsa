use anyhow::{Context, bail};
use kdl::{KdlDocument, KdlNode, KdlValue};

pub trait NodeExtension {
    fn node_name(&self) -> &str;

    fn child_nodes(&self) -> &[KdlNode];

    fn arguments(&self) -> impl Iterator<Item = &KdlValue>;

    fn string_argument(&self) -> anyhow::Result<&str>;

    fn number_argument(&self) -> anyhow::Result<f64>;

    fn number_arguments(&self) -> anyhow::Result<Vec<f64>>;

    fn number_property(&self, key: &str) -> anyhow::Result<f64>;

    fn optional_number_property(&self, key: &str) -> anyhow::Result<Option<f64>>;

    fn string_property(&self, key: &str) -> anyhow::Result<&str>;

    fn bool_property(&self, key: &str) -> anyhow::Result<bool>;
}

impl NodeExtension for KdlNode {
    fn node_name(&self) -> &str {
        self.name().value()
    }

    fn child_nodes(&self) -> &[KdlNode] {
        self.children().map_or(&[], KdlDocument::nodes)
    }

    fn arguments(&self) -> impl Iterator<Item = &KdlValue> {
        self.entries()
            .iter()
            .filter(|entry| entry.name().is_none())
            .map(|entry| entry.value())
    }

    fn string_argument(&self) -> anyhow::Result<&str> {
        self.arguments()
            .next()
            .and_then(KdlValue::as_string)
            .with_context(|| format!("`{}` needs a string argument", self.node_name()))
    }

    fn number_argument(&self) -> anyhow::Result<f64> {
        self.arguments()
            .next()
            .and_then(as_number)
            .with_context(|| format!("`{}` needs a number argument", self.node_name()))
    }

    fn number_arguments(&self) -> anyhow::Result<Vec<f64>> {
        self.arguments()
            .map(|value| as_number(value).with_context(|| format!("`{}` arguments must be numbers", self.node_name())))
            .collect()
    }

    fn number_property(&self, key: &str) -> anyhow::Result<f64> {
        self.optional_number_property(key)?
            .with_context(|| format!("`{}` needs a number property `{key}`", self.node_name()))
    }

    fn optional_number_property(&self, key: &str) -> anyhow::Result<Option<f64>> {
        match self.get(key) {
            None => Ok(None),
            Some(value) => match as_number(value) {
                Some(number) => Ok(Some(number)),
                None => bail!("`{}` property `{key}` must be a number", self.node_name()),
            },
        }
    }

    fn string_property(&self, key: &str) -> anyhow::Result<&str> {
        self.get(key)
            .and_then(KdlValue::as_string)
            .with_context(|| format!("`{}` needs a string property `{key}`", self.node_name()))
    }

    fn bool_property(&self, key: &str) -> anyhow::Result<bool> {
        match self.get(key) {
            None => Ok(false),
            Some(value) => value
                .as_bool()
                .with_context(|| format!("`{}` property `{key}` must be #true or #false", self.node_name())),
        }
    }
}

fn as_number(value: &KdlValue) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|integer| integer as f64))
}
