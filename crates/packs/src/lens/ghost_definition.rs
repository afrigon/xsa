use std::path::PathBuf;

use anyhow::Context;
use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

pub struct GhostDefinition {
    // 0 on the light, 1 at the screen center, beyond 1 past it.
    pub position: f32,
    // Angular diameter.
    pub size_degrees: f32,
    pub texture: PathBuf,
}

impl FromNode for GhostDefinition {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<GhostDefinition> {
        let texture = node
            .child_nodes()
            .iter()
            .find(|child| child.node_name() == "texture")
            .context("`ghost` needs a `texture` child")?;
        let id = context.parse_id(texture.string_argument()?)?;

        Ok(GhostDefinition {
            position: node.number_property("position")? as f32,
            size_degrees: node.number_property("size")? as f32,
            texture: context.stack.texture(&id)?.to_path_buf(),
        })
    }
}
