use super::forward_pipeline::SPECIALIZATION_CONSTANTS;
use super::terrain_variant::TerrainVariant;

// What a forward pipeline fixes about its family's geometry.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(in crate::renderer) enum GeometryVariant {
    Terrain(TerrainVariant),
    Mesh,
}

impl GeometryVariant {
    pub fn specialization_constants(self) -> [u32; SPECIALIZATION_CONSTANTS] {
        match self {
            GeometryVariant::Terrain(terrain) => terrain.specialization_constants(),
            GeometryVariant::Mesh => [0; SPECIALIZATION_CONSTANTS],
        }
    }
}
