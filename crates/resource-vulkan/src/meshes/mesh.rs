use std::path::Path;

use ash::vk;
use memory_vulkan::{Buffer, GpuUpload, MemoryLocation};
use resource_loader::mesh::{BoundingBox, MeshData, Submesh};

use crate::meshes::PendingMesh;
use crate::{LoadContext, MemoryUsage, PendingResource, ResourceKind};

/// A mesh in GPU memory: its vertices (`MeshVertex`), its `u32` indices, the submesh each node draws and the box
/// enclosing them in the model's frame.
pub struct Mesh {
    vertices: Buffer,
    indices: Buffer,
    submeshes: Vec<Submesh>,
    bounding_box: BoundingBox,
}

impl Mesh {
    pub(crate) fn new(vertices: Buffer, indices: Buffer, submeshes: Vec<Submesh>, bounding_box: BoundingBox) -> Mesh {
        Mesh {
            vertices,
            indices,
            submeshes,
            bounding_box,
        }
    }

    pub fn vertices(&self) -> &Buffer {
        &self.vertices
    }

    pub fn indices(&self) -> &Buffer {
        &self.indices
    }

    pub fn submeshes(&self) -> &[Submesh] {
        &self.submeshes
    }

    pub fn bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    fn upload<T: Copy>(
        context: &LoadContext,
        name: &str,
        usage: vk::BufferUsageFlags,
        values: &[T],
    ) -> anyhow::Result<GpuUpload<Buffer>> {
        let size = size_of_val(values);
        let buffer = Buffer::new(
            context.memory,
            &format!("{} {name}", context.name),
            size as u64,
            usage | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuOnly,
            &context.transfer.sharing(),
        )?;
        // `T` is a plain `#[repr(C)]` value or a primitive, so its bytes are its GPU representation.
        let bytes = unsafe { std::slice::from_raw_parts(values.as_ptr().cast::<u8>(), size) }.to_vec();

        Ok(context.transfer.upload_buffer(buffer, bytes, context.priority).into())
    }
}

impl ResourceKind for Mesh {
    type Options = ();

    fn load(context: &LoadContext, path: &Path, _: &()) -> anyhow::Result<Box<dyn PendingResource<Mesh>>> {
        let mesh = MeshData::read(path)?;
        let vertices = Mesh::upload(context, "vertices", vk::BufferUsageFlags::VERTEX_BUFFER, &mesh.vertices)?;
        let indices = Mesh::upload(context, "indices", vk::BufferUsageFlags::INDEX_BUFFER, &mesh.indices)?;

        Ok(Box::new(PendingMesh::new(
            vertices,
            indices,
            mesh.submeshes,
            mesh.bounding_box,
        )))
    }

    fn memory(&self) -> MemoryUsage {
        MemoryUsage {
            video: self.vertices.size() + self.indices.size(),
            ram: 0,
        }
    }
}
