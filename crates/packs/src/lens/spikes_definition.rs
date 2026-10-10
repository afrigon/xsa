use kdl::KdlNode;

use super::SpikesShape;
use crate::{FromNode, NodeExtension, ParseContext};

pub struct SpikesDefinition {
    pub intensity: f32,
    pub length_degrees: f32,
    pub shape: SpikesShape,
}

impl FromNode for SpikesDefinition {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<SpikesDefinition> {
        Ok(SpikesDefinition {
            intensity: node.number_property("intensity")? as f32,
            length_degrees: node.number_property("length")? as f32,
            shape: SpikesShape::from_node(node, context)?,
        })
    }
}
