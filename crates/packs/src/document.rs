mod node_extension;

pub use node_extension::NodeExtension;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use kdl::{KdlDocument, KdlNode};

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
