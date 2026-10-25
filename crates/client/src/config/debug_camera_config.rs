use xsa_config::{ConfigSection, ConfigVisitor, NumberRange};

#[derive(Clone, Debug, PartialEq)]
pub struct DebugCameraConfig {
    // Meters per second; scrolling in the debug camera doubles or halves it.
    pub speed: f32,
}

impl ConfigSection for DebugCameraConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.number("speed", &mut self.speed, NumberRange::at_least(0.0));
    }
}
