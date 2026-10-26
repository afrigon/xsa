use ash::vk;
use glam::Mat4;

use super::GpuData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct MeshDrawData {
    // Places the drawn node of the mesh in the model.
    pub model_from_node: Mat4,
    // The first of the draw's `MeshInstanceData`.
    pub instances: vk::DeviceAddress,
    pub padding: [u32; 2],
}

const _: () = assert!(size_of::<MeshDrawData>() == 80);

unsafe impl GpuData for MeshDrawData {}
