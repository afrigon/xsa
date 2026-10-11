use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

// A lens passes light arriving at an angle θ off its axis in proportion to cos^exponent θ: 4 for an ideal simple lens,
// less for wide-angle designs built to keep their corners bright.
pub struct VignettingDefinition {
    pub exponent: f32,
}

impl FromNode for VignettingDefinition {
    fn from_node(node: &KdlNode, _context: &ParseContext) -> anyhow::Result<VignettingDefinition> {
        Ok(VignettingDefinition {
            exponent: node.number_property("exponent")? as f32,
        })
    }
}
