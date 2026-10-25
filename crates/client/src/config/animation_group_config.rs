use xsa_config::{ConfigSection, ConfigVisitor, NumberRange};

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationGroupConfig {
    pub enabled: bool,
    pub duration_scale: f32,
}

impl AnimationGroupConfig {
    // Off, animations finish at once.
    pub fn effective_duration_scale(&self) -> f32 {
        if self.enabled { self.duration_scale } else { 0.0 }
    }
}

impl ConfigSection for AnimationGroupConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
        visitor.number("duration-scale", &mut self.duration_scale, NumberRange::at_least(0.0));
    }
}
