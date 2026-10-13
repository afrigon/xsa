use std::path::PathBuf;

use kdl::KdlNode;

use crate::{NodeExtension, ParseContext};

// Hapke 2012's parameters other than the albedo: the particulate medium texture (b, c, θ̄ in degrees, porosity K) and
// the opposition texture (B_S0, h_S, B_C0, h_C).
pub struct HapkeDefinition {
    pub medium: PathBuf,
    pub opposition: PathBuf,
}

impl HapkeDefinition {
    // None when a texture is missing: the surface is then shaded without Hapke.
    pub fn parse(node: &KdlNode, context: &ParseContext) -> anyhow::Result<Option<HapkeDefinition>> {
        let texture = |key: &str| context.available_texture(node.string_property(key)?);
        let (Some(medium), Some(opposition)) = (texture("medium-texture")?, texture("opposition-texture")?) else {
            return Ok(None);
        };

        Ok(Some(HapkeDefinition { medium, opposition }))
    }
}
