use glam::{Mat3, Mat4, Vec3, Vec4};

use super::texture::{CubeMapHandle, TextureHandle};

const NO_TEXTURE: u32 = u32::MAX;

macro_rules! shaders {
    ($($variant:ident => $file:literal),+ $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Shader {
            $($variant),+
        }

        impl Shader {
            pub const ALL: &[Shader] = &[$(Shader::$variant),+];

            pub fn index(self) -> usize {
                self as usize
            }

            pub fn path(self) -> &'static str {
                match self {
                    $(Shader::$variant => $file),+
                }
            }
        }
    };
}

shaders! {
    Lit => "lit",
    Planet => "planet",
    Emissive => "emissive",
    Normals => "normals",
    Depth => "depth",
    Triangles => "triangles",
    Lighting => "lighting",
    Skybox => "skybox",
}

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

// Hapke reflectance parameters of a particulate surface (regolith, frost). The scatter texture holds the
// single-scattering albedo w (red), the phase-function lobe width b (green) and lobe balance (c + 1) / 2 (blue); the
// surge texture holds the opposition-surge amplitudes and widths, B_S0 / 2, h_S, B_C0 / 2 and h_C.
#[derive(Clone, Copy)]
pub struct HapkeParameters {
    pub scatter: TextureHandle,
    pub surge: TextureHandle,
    pub porosity: f32,
    pub roughness: f32,
    pub blend: f32,
    pub light_boost: f32,
    pub gamma_boost: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShadingModel {
    HapkeSol,
    HapkeTextbook,
    Lambert,
}

impl ShadingModel {
    pub fn next(self) -> ShadingModel {
        match self {
            ShadingModel::HapkeSol => ShadingModel::HapkeTextbook,
            ShadingModel::HapkeTextbook => ShadingModel::Lambert,
            ShadingModel::Lambert => ShadingModel::HapkeSol,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ShadingModel::HapkeSol => "Hapke (Sol)",
            ShadingModel::HapkeTextbook => "Hapke (textbook)",
            ShadingModel::Lambert => "Lambert",
        }
    }

    // Matches the shading model constants in planet.slang.
    pub(super) fn shader_id(self) -> u32 {
        match self {
            ShadingModel::HapkeSol => 0,
            ShadingModel::HapkeTextbook => 1,
            ShadingModel::Lambert => 2,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct HapkeData {
    scatter_texture: u32,
    surge_texture: u32,
    porosity: f32,
    roughness: f32,
    blend: f32,
    light_boost: f32,
    gamma_boost: f32,
    padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct MaterialData {
    color: Vec4,
    transform: Mat4,
    texture: u32,
    normal_texture: u32,
    emissive_texture: u32,
    luminance: f32,
    hapke: HapkeData,
}

const _: () = assert!(size_of::<MaterialData>() == 128);

impl Material {
    pub fn shader(&self) -> Shader {
        match self {
            Material::Lit { .. } => Shader::Lit,
            Material::Planet { .. } => Shader::Planet,
            Material::Emissive { .. } => Shader::Emissive,
            Material::Skybox { .. } => Shader::Skybox,
        }
    }

    pub(super) fn gpu_data(&self) -> MaterialData {
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
