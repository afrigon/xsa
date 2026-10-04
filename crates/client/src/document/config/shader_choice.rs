use super::ConfigChoice;
use crate::renderer::Shader;

// No override renders every object with its own material's shader.
impl ConfigChoice for Option<Shader> {
    const ALL: &'static [Self] = &[
        None,
        Some(Shader::Normals),
        Some(Shader::Depth),
        Some(Shader::Triangles),
        Some(Shader::Lighting),
    ];

    fn config_name(self) -> &'static str {
        match self {
            Some(Shader::Normals) => "normals",
            Some(Shader::Depth) => "depth",
            Some(Shader::Triangles) => "triangles",
            Some(Shader::Lighting) => "lighting",
            _ => "lit",
        }
    }
}
