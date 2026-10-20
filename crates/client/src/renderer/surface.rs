mod biome_surfaces;
mod surface_class;
mod surface_textures;

pub use biome_surfaces::BiomeSurfaces;
pub use surface_class::SurfaceClass;
pub use surface_textures::SurfaceTextures;

pub(in crate::renderer) const SURFACE_CLASSES: usize = 2;
// Each surface is projected along the body frame's three axes.
pub(in crate::renderer) const PROJECTION_AXES: usize = 3;
// Surfaces repeat at this many sizes, each `SURFACE_LEVEL_GROWTH` times the last, pixels picking theirs by footprint.
// Must match `surfaceLevels` and `surfaceLevelGrowth` in terrain/surface/surface-data.slang.
pub(in crate::renderer) const SURFACE_LEVELS: usize = 12;
pub(in crate::renderer) const SURFACE_LEVEL_GROWTH: f64 = 3.0;
// The distance between neighbouring hexagonal cells' centres, in repeats of the texture.
pub(in crate::renderer) const HEX_CELL_SIZE: f64 = 1.0;
