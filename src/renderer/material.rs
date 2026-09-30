use glam::{Mat3, Mat4, Vec3, Vec4};

use super::texture::CubeMapHandle;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shader {
    Lit,
    Emissive,
    Normals,
    Depth,
    Triangles,
    Lighting,
    Skybox,
}

impl Shader {
    pub const ALL: [Shader; 7] = [
        Shader::Lit,
        Shader::Emissive,
        Shader::Normals,
        Shader::Depth,
        Shader::Triangles,
        Shader::Lighting,
        Shader::Skybox,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn spirv(self) -> &'static [u8] {
        match self {
            Shader::Lit => include_bytes!(concat!(env!("OUT_DIR"), "/lit.spv")),
            Shader::Emissive => include_bytes!(concat!(env!("OUT_DIR"), "/emissive.spv")),
            Shader::Normals => include_bytes!(concat!(env!("OUT_DIR"), "/normals.spv")),
            Shader::Depth => include_bytes!(concat!(env!("OUT_DIR"), "/depth.spv")),
            Shader::Triangles => include_bytes!(concat!(env!("OUT_DIR"), "/triangles.spv")),
            Shader::Lighting => include_bytes!(concat!(env!("OUT_DIR"), "/lighting.spv")),
            Shader::Skybox => include_bytes!(concat!(env!("OUT_DIR"), "/skybox.spv")),
        }
    }
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
