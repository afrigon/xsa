use super::{Shader, ShadingModel, Tonemapper};

// Fraction of all light redistributed into the glow, like the scatter of a real lens.
const INITIAL_BLOOM_STRENGTH: f32 = 0.02;

pub struct RenderSettings {
    pub shader_override: Option<Shader>,
    pub wireframe: bool,
    pub tonemapper: Tonemapper,
    pub shading_model: ShadingModel,
    pub bloom_enabled: bool,
    pub bloom_strength: f32,
}

impl RenderSettings {
    pub fn cycle_shading_model(&mut self) -> ShadingModel {
        self.shading_model = self.shading_model.next();
        self.shading_model
    }

    pub fn cycle_tonemapper(&mut self) -> Tonemapper {
        self.tonemapper = self.tonemapper.next();
        self.tonemapper
    }

    pub fn toggle_bloom(&mut self) -> bool {
        self.bloom_enabled = !self.bloom_enabled;
        self.bloom_enabled
    }

    pub fn adjust_bloom_strength(&mut self, stops: f32) -> f32 {
        self.bloom_strength *= stops.exp2();
        self.bloom_strength
    }

    pub fn effective_bloom_strength(&self) -> f32 {
        if self.bloom_enabled { self.bloom_strength } else { 0.0 }
    }
}

impl Default for RenderSettings {
    fn default() -> Self {
        RenderSettings {
            shader_override: None,
            wireframe: false,
            tonemapper: Tonemapper::Agx,
            shading_model: ShadingModel::HapkeSol,
            bloom_enabled: true,
            bloom_strength: INITIAL_BLOOM_STRENGTH,
        }
    }
}
