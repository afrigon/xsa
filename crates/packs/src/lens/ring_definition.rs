use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

pub struct RingDefinition {
    pub radius: f32,
    pub width: f32,
    pub dispersion: f32,
    pub streaks: u32,
    pub spots: f32,
}

impl FromNode for RingDefinition {
    fn from_node(node: &KdlNode, _context: &ParseContext) -> anyhow::Result<RingDefinition> {
        Ok(RingDefinition {
            radius: node.number_property("radius")? as f32,
            width: node.number_property("width")? as f32,
            dispersion: node.number_property("dispersion")? as f32,
            streaks: node.optional_number_property("streaks")?.unwrap_or(0.0) as u32,
            spots: node.optional_number_property("spots")?.unwrap_or(0.0) as f32,
        })
    }
}
