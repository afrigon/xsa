use crate::renderer::{Shader, ShadingModel};

#[derive(Clone, Debug, PartialEq)]
pub struct DebugConfig {
    pub shader_override: Option<Shader>,
    pub shading_model: ShadingModel,
    pub wireframe: bool,
    pub animation_duration_scale: f32,
}
