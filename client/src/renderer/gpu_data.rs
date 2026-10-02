use ash::vk;
use glam::{Mat4, Vec2, Vec4};

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct FrameData {
    pub view_projection: Mat4,
    pub world_from_clip: Mat4,
    pub sun_position: Vec4,
    pub viewport_size: Vec2,
    pub point_quad_size: f32,
    pub exposure: f32,
    pub hdr_texture: u32,
    pub tonemapper: u32,
    pub padding: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct ObjectData {
    pub world_from_model: Mat4,
    pub point_diameter: f32,
    pub point_intensity: f32,
    pub padding: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct PushConstants {
    pub frame: vk::DeviceAddress,
    pub objects: vk::DeviceAddress,
    pub materials: vk::DeviceAddress,
    pub object_index: u32,
    pub material_index: u32,
}

const _: () = assert!(size_of::<FrameData>() == 176);
const _: () = assert!(size_of::<ObjectData>() == 80);
const _: () = assert!(size_of::<PushConstants>() == 32);

pub(super) fn as_bytes<T: Copy>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((value as *const T).cast::<u8>(), size_of::<T>()) }
}
