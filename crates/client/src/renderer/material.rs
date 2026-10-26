mod hapke_data;
mod hapke_material;
mod material_buffer;
mod material_data;
mod shader;
mod shader_data;
mod skybox_data;
mod skybox_material;
mod terrain_emissive_data;
mod terrain_emissive_material;
mod terrain_hapke_data;
mod terrain_hapke_material;
mod terrain_lambert_data;
mod terrain_lambert_material;

pub use hapke_material::HapkeMaterial;
pub(super) use material_buffer::MaterialBuffer;
pub use material_data::MaterialData;
pub use shader::Shader;
pub use skybox_material::SkyboxMaterial;
pub use terrain_emissive_material::TerrainEmissiveMaterial;
pub use terrain_hapke_material::TerrainHapkeMaterial;
pub use terrain_lambert_material::TerrainLambertMaterial;

use resource_vulkan::{ResourceHandle, Texture};

pub(in crate::renderer) const NO_TEXTURE: u32 = u32::MAX;

// A shader's parameters, read through the draw's material address.
pub trait Material {
    // None while it cannot be drawn, such as before a texture it cannot do without is loaded.
    fn gpu_data(&self) -> Option<MaterialData>;
}

fn texture_index(texture: Option<&ResourceHandle<Texture>>) -> u32 {
    texture.and_then(ResourceHandle::get).map_or(NO_TEXTURE, Texture::index)
}
