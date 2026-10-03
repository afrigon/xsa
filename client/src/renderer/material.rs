mod hapke_data;
mod hapke_parameters;
mod material_data;
mod shader;
mod shading_model;

pub use hapke_parameters::HapkeParameters;
pub(super) use material_data::MaterialData;
pub use shader::Shader;
pub use shading_model::ShadingModel;

use glam::{Mat3, Mat4, Vec3, Vec4};

use super::{CubeMapHandle, TextureHandle};
use hapke_data::HapkeData;

const NO_TEXTURE: u32 = u32::MAX;

#[derive(Clone, Copy)]
pub enum Material {
    Lit {
        base_color: Vec3,
    },
    Planet {
        color: TextureHandle,
        normal: Option<TextureHandle>,
        emissive: Option<TextureHandle>,
        emissive_luminance: f32,
        hapke: Option<HapkeParameters>,
    },
    Emissive {
        luminance: Vec3,
    },
    Skybox {
        cube_map: CubeMapHandle,
        orientation: Mat3,
        luminance: f32,
    },
}

impl Material {
    pub fn shader(&self) -> Shader {
        match self {
            Material::Lit { .. } => Shader::Lit,
            Material::Planet { .. } => Shader::Planet,
            Material::Emissive { .. } => Shader::Emissive,
            Material::Skybox { .. } => Shader::Skybox,
        }
    }

    pub(in crate::renderer) fn gpu_data(&self) -> MaterialData {
        let mut data = MaterialData {
            color: Vec4::ONE,
            transform: Mat4::IDENTITY,
            texture: NO_TEXTURE,
            normal_texture: NO_TEXTURE,
            emissive_texture: NO_TEXTURE,
            luminance: 0.0,
            hapke: HapkeData {
                scatter_texture: NO_TEXTURE,
                surge_texture: NO_TEXTURE,
                porosity: 0.0,
                roughness: 0.0,
                blend: 0.0,
                light_boost: 0.0,
                gamma_boost: 0.0,
                padding: 0,
            },
        };
        let index = |texture: Option<TextureHandle>| texture.map_or(NO_TEXTURE, TextureHandle::index);

        match *self {
            Material::Lit { base_color } => data.color = base_color.extend(1.0),
            Material::Planet {
                color,
                normal,
                emissive,
                emissive_luminance,
                hapke,
            } => {
                data.texture = color.index();
                data.normal_texture = index(normal);
                data.emissive_texture = index(emissive);
                data.luminance = emissive_luminance;

                if let Some(hapke) = hapke {
                    data.hapke = HapkeData {
                        scatter_texture: hapke.scatter.index(),
                        surge_texture: hapke.surge.index(),
                        porosity: hapke.porosity,
                        roughness: hapke.roughness,
                        blend: hapke.blend,
                        light_boost: hapke.light_boost,
                        gamma_boost: hapke.gamma_boost,
                        padding: 0,
                    };
                }
            }
            Material::Emissive { luminance } => data.color = luminance.extend(1.0),
            Material::Skybox {
                cube_map,
                orientation,
                luminance,
            } => {
                data.transform = Mat4::from_mat3(orientation);
                data.texture = cube_map.index();
                data.luminance = luminance;
            }
        }

        data
    }
}
