use std::path::PathBuf;

use anyhow::{Context, ensure};
use glam::Vec3;
use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

const UNTINTED: Vec3 = Vec3::ONE;

pub struct GhostDefinition {
    // 0 on the light, 1 at the screen center, beyond 1 past it.
    pub position: f32,
    // Angular diameter.
    pub size_degrees: f32,
    // Red moves outward along the axis and blue inward, each by this fraction of the ghost's size.
    pub dispersion: f32,
    // Linear multiplier per color channel.
    pub tint: Vec3,
    pub texture: PathBuf,
}

impl FromNode for GhostDefinition {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<GhostDefinition> {
        let child = |name: &str| node.child_nodes().iter().find(|child| child.node_name() == name);
        let texture = child("texture").context("`ghost` needs a `texture` child")?;
        let id = context.parse_id(texture.string_argument()?)?;
        let tint = match child("tint") {
            Some(tint) => {
                let components = tint.number_arguments()?;
                ensure!(
                    components.len() == 3,
                    "`tint` needs three components: red green blue, linear"
                );
                Vec3::new(components[0] as f32, components[1] as f32, components[2] as f32)
            }
            None => UNTINTED,
        };

        Ok(GhostDefinition {
            position: node.number_property("position")? as f32,
            size_degrees: node.number_property("size")? as f32,
            dispersion: node.optional_number_property("dispersion")?.unwrap_or(0.0) as f32,
            tint,
            texture: context.stack.texture(&id)?.to_path_buf(),
        })
    }
}
