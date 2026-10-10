use kdl::KdlNode;

use super::GhostShape;
use crate::{FromNode, NodeExtension, ParseContext};

const ROUND: f64 = 1.0;

pub struct GhostDefinition {
    // 0 on the star, 1 at the screen center, beyond 1 past it.
    pub position: f32,
    // Fraction of the screen height.
    pub size: f32,
    pub aspect_ratio: f32,
    pub intensity: f32,
    pub shape: GhostShape,
}

impl FromNode for GhostDefinition {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<GhostDefinition> {
        Ok(GhostDefinition {
            position: node.number_property("position")? as f32,
            size: node.number_property("size")? as f32,
            aspect_ratio: node.optional_number_property("aspect-ratio")?.unwrap_or(ROUND) as f32,
            intensity: node.number_property("intensity")? as f32,
            shape: GhostShape::from_node(node, context)?,
        })
    }
}
