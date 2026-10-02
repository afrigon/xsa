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

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct MaterialData {
    color: Vec4,
    transform: Mat4,
    texture: u32,
    normal_texture: u32,
    emissive_texture: u32,
    luminance: f32,
}

const _: () = assert!(size_of::<MaterialData>() == 96);

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
        };
        let index = |texture: Option<TextureHandle>| texture.map_or(NO_TEXTURE, TextureHandle::index);
        match *self {
            Material::Lit { base_color } => data.color = base_color.extend(1.0),
            Material::Planet {
                color,
                normal,
                emissive,
                emissive_luminance,
            } => {
                data.texture = color.index();
                data.normal_texture = index(normal);
                data.emissive_texture = index(emissive);
                data.luminance = emissive_luminance;
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
