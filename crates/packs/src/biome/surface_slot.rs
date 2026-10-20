use crate::{Id, NodeExtension, PackStack, SurfaceDefinition};

// One class of a biome's ground: its surface, the feature size (m) of the seeded noise choosing between the surface's
// variants, and how sharply the higher of two blended materials shows through.
pub struct SurfaceSlot {
    pub surface: SurfaceDefinition,
    pub variant_noise: f32,
    pub height_contrast: f32,
}

impl SurfaceSlot {
    // None while the surface's textures are missing.
    pub(super) fn load(stack: &PackStack, node: &kdl::KdlNode, biome: &Id) -> anyhow::Result<Option<SurfaceSlot>> {
        let surface = Id::parse(node.string_argument()?, &biome.namespace)?;
        let Some(surface) = SurfaceDefinition::load(stack, &surface)? else {
            return Ok(None);
        };

        Ok(Some(SurfaceSlot {
            surface,
            variant_noise: node.number_property("variant-noise")? as f32,
            height_contrast: node.number_property("height-contrast")? as f32,
        }))
    }
}
