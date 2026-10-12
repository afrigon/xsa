use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

// A lens passes light arriving at an angle θ off its axis in proportion to cos^exponent θ: 4 for an ideal simple lens,
// less for wide-angle designs built to keep their corners bright.
pub struct OffAxisFalloffDefinition {
    pub exponent: f32,
}

impl FromNode for OffAxisFalloffDefinition {
    fn from_node(node: &KdlNode, _context: &ParseContext) -> anyhow::Result<OffAxisFalloffDefinition> {
        Ok(OffAxisFalloffDefinition {
            exponent: node.number_property("exponent")? as f32,
        })
    }
}
