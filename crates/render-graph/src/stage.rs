use ash::vk;

/// The shader stage a step reads or writes a resource from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Vertex,
    Fragment,
    Compute,
}

impl Stage {
    pub(crate) fn pipeline_stages(self) -> vk::PipelineStageFlags2 {
        match self {
            Stage::Vertex => vk::PipelineStageFlags2::VERTEX_SHADER,
            Stage::Fragment => vk::PipelineStageFlags2::FRAGMENT_SHADER,
            Stage::Compute => vk::PipelineStageFlags2::COMPUTE_SHADER,
        }
    }
}
