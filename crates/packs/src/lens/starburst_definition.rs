use std::path::PathBuf;

use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

// The starburst around a light, as a log-polar texture: rows are angles from the light on a log scale from `inner` to
// `extent` degrees, columns directions around it.
pub struct StarburstDefinition {
    pub texture: PathBuf,
    pub inner_degrees: f32,
    pub extent_degrees: f32,
}

impl FromNode for StarburstDefinition {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<StarburstDefinition> {
        let id = context.parse_id(node.string_property("texture")?)?;

        Ok(StarburstDefinition {
            texture: context.stack.texture(&id)?.to_path_buf(),
            inner_degrees: node.number_property("inner")? as f32,
            extent_degrees: node.number_property("extent")? as f32,
        })
    }
}
