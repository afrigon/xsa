use super::Shader;
use crate::renderer::gpu_data::GpuData;

// The parameters one shader reads through the draw's material address.
pub(super) trait ShaderData: GpuData {
    const SHADER: Shader;
}
