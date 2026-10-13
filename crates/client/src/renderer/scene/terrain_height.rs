use crate::renderer::TextureHandle;

// Heights are meters relative to the body's radius, signed.
pub struct TerrainHeight {
    pub texture: TextureHandle,
    pub minimum: f64,
    pub maximum: f64,
}
