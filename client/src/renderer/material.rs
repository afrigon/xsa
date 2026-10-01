use glam::{Mat3, Mat4, Vec3, Vec4};

use super::texture::CubeMapHandle;

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

            pub fn spirv(self) -> &'static [u8] {
                match self {
                    $(Shader::$variant => include_bytes!(concat!(env!("OUT_DIR"), "/", $file, ".spv"))),+
                }
            }
        }
    };
}

shaders! {
    Lit => "lit",
    Emissive => "emissive",
    Normals => "normals",
    Depth => "depth",
    Triangles => "triangles",
    Lighting => "lighting",
    Skybox => "skybox",
}

#[derive(Clone, Copy)]
pub enum Material {
    Lit { base_color: Vec3 },
    Emissive { color: Vec3 },
    Skybox { cube_map: CubeMapHandle, orientation: Mat3 },
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct MaterialData {
    color: Vec4,
    transform: Mat4,
    texture_index: u32,
    padding: [u32; 3],
}

const _: () = assert!(size_of::<MaterialData>() == 96);

impl Material {
    pub fn shader(&self) -> Shader {
        match self {
            Material::Lit { .. } => Shader::Lit,
            Material::Emissive { .. } => Shader::Emissive,
            Material::Skybox { .. } => Shader::Skybox,
        }
    }

    pub(super) fn gpu_data(&self) -> MaterialData {
        let mut data = MaterialData {
            color: Vec4::ONE,
            transform: Mat4::IDENTITY,
            texture_index: 0,
            padding: [0; 3],
        };
        match *self {
            Material::Lit { base_color } => data.color = base_color.extend(1.0),
            Material::Emissive { color } => data.color = color.extend(1.0),
            Material::Skybox { cube_map, orientation } => {
                data.transform = Mat4::from_mat3(orientation);
                data.texture_index = cube_map.index();
            }
        }
        data
    }
}
