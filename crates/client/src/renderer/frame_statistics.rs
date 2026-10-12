use super::LensLights;

#[derive(Clone, Copy, Default)]
pub struct FrameStatistics {
    pub triangles: u64,
    pub ev100: f32,
    pub anchor_ev100: f32,
    pub star_visibility: f32,
    pub light: f32,
    pub lens_lights: LensLights,
    pub gpu_milliseconds: Option<f32>,
}
