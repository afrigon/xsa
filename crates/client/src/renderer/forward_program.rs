use super::Shader;
use super::geometry::Geometry;

// A material shader specialized for one geometry: one SPIR-V module and one forward pipeline.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(in crate::renderer) struct ForwardProgram {
    pub geometry: Geometry,
    pub shader: Shader,
}

impl ForwardProgram {
    pub fn all() -> impl Iterator<Item = ForwardProgram> {
        Geometry::ALL.iter().flat_map(|geometry| {
            // The skybox is drawn by its own pass.
            Shader::ALL
                .iter()
                .filter(|shader| **shader != Shader::Skybox)
                .map(|shader| ForwardProgram {
                    geometry: *geometry,
                    shader: *shader,
                })
        })
    }

    pub fn path(self) -> String {
        format!("{}-{}", self.shader.path(), self.geometry.path())
    }
}
