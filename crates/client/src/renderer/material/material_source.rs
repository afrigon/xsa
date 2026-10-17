use super::{Material, Shader};
use crate::renderer::gpu_data::GpuData;

// A material of any type, as its buffer sees it: the shader it is drawn with and its parameters' current bytes.
pub(in crate::renderer) trait MaterialSource {
    fn shader(&self) -> Shader;

    // None while it cannot be drawn.
    fn gpu_bytes(&self) -> Option<Vec<u8>>;
}

impl<M: Material> MaterialSource for M {
    fn shader(&self) -> Shader {
        M::SHADER
    }

    fn gpu_bytes(&self) -> Option<Vec<u8>> {
        Some(self.gpu_data()?.as_bytes().to_vec())
    }
}
