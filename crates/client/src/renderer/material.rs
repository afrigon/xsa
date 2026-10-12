mod emissive_data;
mod emissive_material;
mod hapke_data;
mod hapke_parameters;
mod lit_data;
mod lit_material;
mod material_buffer;
mod planet_data;
mod planet_material;
mod shader;
mod shading_model;
mod skybox_data;
mod skybox_material;

pub use emissive_material::EmissiveMaterial;
pub use hapke_parameters::HapkeParameters;
pub use lit_material::LitMaterial;
pub(super) use material_buffer::MaterialBuffer;
pub use planet_material::PlanetMaterial;
pub use shader::Shader;
pub use shading_model::ShadingModel;
pub use skybox_material::SkyboxMaterial;

use super::gpu_data::GpuData;

pub(in crate::renderer) const NO_TEXTURE: u32 = u32::MAX;

// A shader's parameters: `Data` matches the struct its shader reads through the draw's material address.
pub trait Material {
    type Data: GpuData;
    const SHADER: Shader;

    fn gpu_data(&self) -> Self::Data;
}
