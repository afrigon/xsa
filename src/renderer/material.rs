#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Material {
    Lit,
    Emissive,
    Normals,
    Depth,
    Triangles,
    Lighting,
}

impl Material {
    pub const ALL: [Material; 6] = [
        Material::Lit,
        Material::Emissive,
        Material::Normals,
        Material::Depth,
        Material::Triangles,
        Material::Lighting,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn spirv(self) -> &'static [u8] {
        match self {
            Material::Lit => include_bytes!(concat!(env!("OUT_DIR"), "/lit.spv")),
            Material::Emissive => include_bytes!(concat!(env!("OUT_DIR"), "/emissive.spv")),
            Material::Normals => include_bytes!(concat!(env!("OUT_DIR"), "/normals.spv")),
            Material::Depth => include_bytes!(concat!(env!("OUT_DIR"), "/depth.spv")),
            Material::Triangles => include_bytes!(concat!(env!("OUT_DIR"), "/triangles.spv")),
            Material::Lighting => include_bytes!(concat!(env!("OUT_DIR"), "/lighting.spv")),
        }
    }
}
