use anyhow::{Context, ensure};
use glam::Vec3;
use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

pub struct PolygonDefinition {
    pub softness: f32,
    pub tint: Vec3,
}

impl FromNode for PolygonDefinition {
    fn from_node(node: &KdlNode, _context: &ParseContext) -> anyhow::Result<PolygonDefinition> {
        let tint = node
            .child_nodes()
            .iter()
            .find(|child| child.node_name() == "tint")
            .context("`polygon` needs a `tint` child: red green blue, linear")?
            .number_arguments()?;
        ensure!(tint.len() == 3, "`tint` needs three components: red green blue");

        Ok(PolygonDefinition {
            softness: node.number_property("softness")? as f32,
            tint: Vec3::new(tint[0] as f32, tint[1] as f32, tint[2] as f32),
        })
    }
}
