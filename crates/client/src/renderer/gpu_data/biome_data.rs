use super::GpuData;

// A biome's selection of classes, as the terrain programs read it. Mirrored by `BiomeData` in
// terrain/surface/surface-data.slang. Angles and biases are in radians.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::renderer) struct BiomeData {
    pub flat_to_slope_angle: f32,
    pub flat_to_slope_width: f32,
    pub slope_to_steep_angle: f32,
    pub slope_to_steep_width: f32,
    pub handover_noise_amplitude: f32,
    pub convex_bias: f32,
    pub concave_bias: f32,
    pub relief_bias: f32,
}

const _: () = assert!(size_of::<BiomeData>() == 32);

unsafe impl GpuData for BiomeData {}

impl BiomeData {
    pub const NONE: BiomeData = BiomeData {
        flat_to_slope_angle: 0.0,
        flat_to_slope_width: 0.0,
        slope_to_steep_angle: 0.0,
        slope_to_steep_width: 0.0,
        handover_noise_amplitude: 0.0,
        convex_bias: 0.0,
        concave_bias: 0.0,
        relief_bias: 0.0,
    };
}
