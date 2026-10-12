use std::path::PathBuf;

use kdl::KdlNode;

use crate::{NodeExtension, ParseContext};

pub struct HapkeDefinition {
    pub scatter: PathBuf,
    pub surge: PathBuf,
    pub porosity: f32,
    pub roughness_degrees: f32,
    pub blend: f32,
    pub light_boost: f32,
    pub gamma_boost: f32,
}

impl HapkeDefinition {
    // None when a texture is missing: the constants describe the surface only together with its maps.
    pub fn parse(node: &KdlNode, context: &ParseContext) -> anyhow::Result<Option<HapkeDefinition>> {
        let texture = |key: &str| context.available_texture(node.string_property(key)?);
        let number = |key: &str| node.number_property(key).map(|value| value as f32);
        let (Some(scatter), Some(surge)) = (texture("scatter-texture")?, texture("surge-texture")?) else {
            return Ok(None);
        };

        Ok(Some(HapkeDefinition {
            scatter,
            surge,
            porosity: number("porosity")?,
            roughness_degrees: number("roughness")?,
            blend: number("blend")?,
            light_boost: number("light-boost")?,
            gamma_boost: number("gamma-boost")?,
        }))
    }
}
