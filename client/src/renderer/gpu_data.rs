use ash::vk;
use glam::{Mat4, Vec2, Vec4};

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct FrameData {
    pub view_projection: Mat4,
    pub world_from_clip: Mat4,
    pub sun_position: Vec4,
    pub sun_intensity: Vec4,
    pub viewport_size: Vec2,
    pub exposure: f32,
    pub hdr_texture: u32,
    pub tonemapper: u32,
    pub starlight_illuminance: f32,
    pub bloom_texture: u32,
    pub bloom_strength: f32,
    pub shading_model: u32,
    pub padding: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct ObjectData {
    pub world_from_model: Mat4,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct HistogramPushConstants {
    pub frame: vk::DeviceAddress,
    pub histogram: vk::DeviceAddress,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct BloomPushConstants {
    pub source_texture: u32,
    pub source_level: u32,
    pub target_image: u32,
    pub karis_average: u32,
}

const _: () = assert!(size_of::<FrameData>() == 208);
const _: () = assert!(size_of::<ObjectData>() == 64);
const _: () = assert!(size_of::<PushConstants>() == 32);

pub(super) fn as_bytes<T: Copy>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts((value as *const T).cast::<u8>(), size_of::<T>()) }
}
