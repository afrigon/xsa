use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

pub struct GlareDefinition {
    pub intensity: f32,
    pub falloff: f32,
}

impl FromNode for GlareDefinition {
    fn from_node(node: &KdlNode, _context: &ParseContext) -> anyhow::Result<GlareDefinition> {
        Ok(GlareDefinition {
            intensity: node.number_property("intensity")? as f32,
            falloff: node.number_property("falloff")? as f32,
        })
    }
}
