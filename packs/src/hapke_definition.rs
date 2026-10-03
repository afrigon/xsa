use std::path::PathBuf;

use kdl::KdlNode;

use crate::{FromNode, NodeExtension, ParseContext};

pub struct HapkeDefinition {
    pub scatter: PathBuf,
    pub surge: PathBuf,
    pub porosity: f32,
    pub roughness_degrees: f32,
    pub blend: f32,
    pub light_boost: f32,
    pub gamma_boost: f32,
}

impl FromNode for HapkeDefinition {
    fn from_node(node: &KdlNode, context: &ParseContext) -> anyhow::Result<HapkeDefinition> {
        let texture = |key: &str| -> anyhow::Result<PathBuf> {
            let id = context.parse_id(node.string_property(key)?)?;

            Ok(context.stack.texture(&id)?.to_path_buf())
        };
        let number = |key: &str| node.number_property(key).map(|value| value as f32);

        Ok(HapkeDefinition {
            scatter: texture("scatter-texture")?,
            surge: texture("surge-texture")?,
            porosity: number("porosity")?,
            roughness_degrees: number("roughness")?,
            blend: number("blend")?,
            light_boost: number("light-boost")?,
            gamma_boost: number("gamma-boost")?,
        })
    }
}
