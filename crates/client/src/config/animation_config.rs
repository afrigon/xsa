use xsa_config::{ConfigSection, ConfigVisitor};

use super::AnimationGroupConfig;

// A group runs only when it and every animation are enabled, at the product of their scales.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimationConfig {
    pub all: AnimationGroupConfig,
    // Camera transitions and eye adaptation.
    pub camera: AnimationGroupConfig,
    pub ui: AnimationGroupConfig,
    // Time warps easing to their end.
    pub time: AnimationGroupConfig,
}

impl AnimationConfig {
    pub fn camera_duration_scale(&self) -> f32 {
        self.all.effective_duration_scale() * self.camera.effective_duration_scale()
    }

    pub fn ui_duration_scale(&self) -> f32 {
        self.all.effective_duration_scale() * self.ui.effective_duration_scale()
    }

    pub fn time_duration_scale(&self) -> f32 {
        self.all.effective_duration_scale() * self.time.effective_duration_scale()
    }
}

impl ConfigSection for AnimationConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        self.all.visit(visitor);
        visitor.section("camera", &mut self.camera);
        visitor.section("ui", &mut self.ui);
        visitor.section("time", &mut self.time);
    }
}
