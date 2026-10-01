use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use kdl::{KdlDocument, KdlNode, KdlValue};

pub struct Document {
    path: PathBuf,
    document: KdlDocument,
}

impl Document {
    pub fn read(path: &Path) -> anyhow::Result<Document> {
        let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let document = KdlDocument::parse(&text).with_context(|| format!("parsing {}", path.display()))?;
        Ok(Document {
            path: path.to_path_buf(),
            document,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn nodes(&self) -> &[KdlNode] {
        self.document.nodes()
    }

    pub fn node(&self, name: &str) -> anyhow::Result<&KdlNode> {
        self.optional_node(name)
            .with_context(|| format!("{}: missing `{name}`", self.path.display()))
    }

    pub fn optional_node(&self, name: &str) -> Option<&KdlNode> {
        self.document.get(name)
    }
}

pub fn name(node: &KdlNode) -> &str {
    node.name().value()
}

pub fn children(node: &KdlNode) -> &[KdlNode] {
    node.children().map_or(&[], KdlDocument::nodes)
}

pub fn arguments(node: &KdlNode) -> impl Iterator<Item = &KdlValue> {
    node.entries()
        .iter()
        .filter(|entry| entry.name().is_none())
        .map(|entry| entry.value())
}

pub fn string_argument(node: &KdlNode) -> anyhow::Result<&str> {
    arguments(node)
        .next()
        .and_then(KdlValue::as_string)
        .with_context(|| format!("`{}` needs a string argument", name(node)))
}

pub fn number_argument(node: &KdlNode) -> anyhow::Result<f64> {
    arguments(node)
        .next()
        .and_then(as_number)
        .with_context(|| format!("`{}` needs a number argument", name(node)))
}

pub fn number_arguments(node: &KdlNode) -> anyhow::Result<Vec<f64>> {
    arguments(node)
        .map(|value| as_number(value).with_context(|| format!("`{}` arguments must be numbers", name(node))))
        .collect()
}

pub fn number_property(node: &KdlNode, key: &str) -> anyhow::Result<f64> {
    optional_number_property(node, key)?.with_context(|| format!("`{}` needs a number property `{key}`", name(node)))
}

pub fn optional_number_property(node: &KdlNode, key: &str) -> anyhow::Result<Option<f64>> {
    match node.get(key) {
        None => Ok(None),
        Some(value) => match as_number(value) {
            Some(number) => Ok(Some(number)),
            None => bail!("`{}` property `{key}` must be a number", name(node)),
        },
    }
}

pub fn string_property<'a>(node: &'a KdlNode, key: &str) -> anyhow::Result<&'a str> {
    node.get(key)
        .and_then(KdlValue::as_string)
        .with_context(|| format!("`{}` needs a string property `{key}`", name(node)))
}

pub fn bool_property(node: &KdlNode, key: &str) -> anyhow::Result<bool> {
    match node.get(key) {
        None => Ok(false),
        Some(value) => value
            .as_bool()
            .with_context(|| format!("`{}` property `{key}` must be #true or #false", name(node))),
    }
}

fn as_number(value: &KdlValue) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|integer| integer as f64))
}
