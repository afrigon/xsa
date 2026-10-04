#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct HapkeData {
    pub scatter_texture: u32,
    pub surge_texture: u32,
    pub porosity: f32,
    pub roughness: f32,
    pub blend: f32,
    pub light_boost: f32,
    pub gamma_boost: f32,
    pub padding: u32,
}
