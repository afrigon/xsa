use glam::Vec3;

use super::hapke_data::HapkeData;
use super::{Material, MaterialData};

// A mesh shaded with Hapke 2012, every parameter constant over it; each instance tints its base colour.
pub struct HapkeMaterial {
    pub base_color: Vec3,
    pub lobe_width: f32,
    pub lobe_balance: f32,
    // Radians.
    pub macroscopic_roughness: f32,
    pub porosity: f32,
    pub shadow_hiding_amplitude: f32,
    pub shadow_hiding_width: f32,
    pub backscatter_amplitude: f32,
    pub backscatter_width: f32,
}

impl Material for HapkeMaterial {
    fn gpu_data(&self) -> Option<MaterialData> {
        Some(MaterialData::new(HapkeData {
            base_color: self.base_color,
            lobe_width: self.lobe_width,
            lobe_balance: self.lobe_balance,
            macroscopic_roughness: self.macroscopic_roughness,
            porosity: self.porosity,
            shadow_hiding_amplitude: self.shadow_hiding_amplitude,
            shadow_hiding_width: self.shadow_hiding_width,
            backscatter_amplitude: self.backscatter_amplitude,
            backscatter_width: self.backscatter_width,
            padding: 0,
        }))
    }
}
