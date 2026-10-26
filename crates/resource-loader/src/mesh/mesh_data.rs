use std::path::Path;

use anyhow::{Context, ensure};

use super::{BoundingBox, GlbLoader, MeshVertex, Submesh};

/// A decoded mesh: one vertex buffer and one index buffer shared by its submeshes, one per node, whose indices already
/// point into the shared vertices, and the box enclosing every node where it stands in the model.
pub struct MeshData {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub submeshes: Vec<Submesh>,
    pub bounding_box: BoundingBox,
}

impl MeshData {
    pub fn read(path: &Path) -> anyhow::Result<MeshData> {
        let extension = path.extension().and_then(|extension| extension.to_str());
        ensure!(
            extension == Some(GlbLoader::EXTENSION),
            "{}: meshes are binary glTF files",
            path.display()
        );

        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;

        GlbLoader::load(&bytes).with_context(|| format!("{}", path.display()))
    }
}
