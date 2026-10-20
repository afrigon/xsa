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
        Some(Shader::Motion),
        Some(Shader::Lod),
        Some(Shader::Scale),
        Some(Shader::TerrainLevels),
        Some(Shader::TerrainFallback),
    ];

    fn config_name(self) -> &'static str {
        match self {
            Some(Shader::Normals) => "normals",
            Some(Shader::Depth) => "depth",
            Some(Shader::Triangles) => "triangles",
            Some(Shader::Lighting) => "lighting",
            Some(Shader::Motion) => "motion",
            Some(Shader::Lod) => "lod",
            Some(Shader::Scale) => "scale",
            Some(Shader::TerrainLevels) => "terrain-levels",
            Some(Shader::TerrainFallback) => "terrain-fallback",
            _ => "none",
        }
    }
}
