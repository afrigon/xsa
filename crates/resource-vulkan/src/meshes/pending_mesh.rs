use memory_vulkan::{BindlessTable, Buffer, GpuUpload, TransferPriority};
use resource_loader::mesh::{BoundingBox, Submesh};

use crate::PendingResource;
use crate::meshes::Mesh;

/// A mesh whose vertex and index uploads are queued.
pub struct PendingMesh {
    // All three are taken when the mesh becomes ready.
    vertices: Option<GpuUpload<Buffer>>,
    indices: Option<GpuUpload<Buffer>>,
    submeshes: Option<Vec<Submesh>>,
    bounding_box: BoundingBox,
}

impl PendingMesh {
    pub(crate) fn new(
        vertices: GpuUpload<Buffer>,
        indices: GpuUpload<Buffer>,
        submeshes: Vec<Submesh>,
        bounding_box: BoundingBox,
    ) -> PendingMesh {
        PendingMesh {
            vertices: Some(vertices),
            indices: Some(indices),
            submeshes: Some(submeshes),
            bounding_box,
        }
    }
}

impl PendingResource<Mesh> for PendingMesh {
    /// # Panics
    ///
    /// If polled again after it returned the mesh or an error.
    fn poll(&mut self, _: &BindlessTable) -> anyhow::Result<Option<Mesh>> {
        let (Some(vertices), Some(indices)) = (self.vertices.as_mut(), self.indices.as_mut()) else {
            panic!("a pending mesh is polled until it settles");
        };
        let vertices_ready = vertices.poll()?.is_some();
        let indices_ready = indices.poll()?.is_some();

        if !vertices_ready || !indices_ready {
            return Ok(None);
        }

        let (Some(GpuUpload::Ready(vertices)), Some(GpuUpload::Ready(indices)), Some(submeshes)) =
            (self.vertices.take(), self.indices.take(), self.submeshes.take())
        else {
            unreachable!("polled ready uploads hold their buffers");
        };

        Ok(Some(Mesh::new(vertices, indices, submeshes, self.bounding_box)))
    }

    fn set_priority(&self, priority: TransferPriority) {
        for upload in [&self.vertices, &self.indices].into_iter().flatten() {
            upload.set_priority(priority);
        }
    }
}
