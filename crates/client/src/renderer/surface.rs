mod biome_surfaces;
mod surface_class;
mod surface_scale;
mod surface_textures;

pub use biome_surfaces::BiomeSurfaces;
pub use surface_class::SurfaceClass;
pub use surface_textures::SurfaceTextures;

pub(in crate::renderer) use surface_scale::SurfaceScale;

pub(in crate::renderer) const SURFACE_CLASSES: usize = 3;
// Each surface is projected along the body frame's three axes.
pub(in crate::renderer) const TRIPLANAR_PLANES: usize = 3;
