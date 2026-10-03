use glam::{Mat4, Vec4};

use super::hapke_data::HapkeData;

#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct MaterialData {
    pub(super) color: Vec4,
    pub(super) transform: Mat4,
    pub(super) texture: u32,
    pub(super) normal_texture: u32,
    pub(super) emissive_texture: u32,
    pub(super) luminance: f32,
    pub(super) hapke: HapkeData,
}

const _: () = assert!(size_of::<MaterialData>() == 128);
