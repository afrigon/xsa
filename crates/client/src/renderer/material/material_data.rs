use super::Shader;
use super::shader_data::ShaderData;

// A material's parameters with the shader that reads them, so the two always change together.
#[derive(PartialEq)]
pub struct MaterialData {
    pub(super) shader: Shader,
    pub(super) bytes: Vec<u8>,
}

impl MaterialData {
    pub(super) fn new<D: ShaderData>(data: D) -> MaterialData {
        MaterialData {
            shader: D::SHADER,
            bytes: data.as_bytes().to_vec(),
        }
    }
}
