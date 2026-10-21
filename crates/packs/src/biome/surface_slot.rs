use anyhow::ensure;

use crate::{Id, NodeExtension, PackStack, SurfaceDefinition};

// One class of a biome's ground: its surface, and how sharply the higher of two blended materials shows through: from
// 0, a cross-fade over the whole displacement range, toward 1, only the higher material.
pub struct SurfaceSlot {
    pub surface: SurfaceDefinition,
    pub height_contrast: f32,
}

impl SurfaceSlot {
    // None while the surface's textures are missing.
    pub(super) fn load(stack: &PackStack, node: &kdl::KdlNode, biome: &Id) -> anyhow::Result<Option<SurfaceSlot>> {
        let surface = Id::parse(node.string_argument()?, &biome.namespace)?;
        let Some(surface) = SurfaceDefinition::load(stack, &surface)? else {
            return Ok(None);
        };

        let height_contrast = node.number_property("height-contrast")? as f32;
        ensure!(
            (0.0..1.0).contains(&height_contrast),
            "`height-contrast` runs from 0 up to, but not including, 1"
        );

        Ok(Some(SurfaceSlot {
            surface,
            height_contrast,
        }))
    }
}
