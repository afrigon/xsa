use xsa_config::{ConfigSection, ConfigVisitor};

use super::DebugCameraConfig;

#[derive(Clone, Debug, PartialEq)]
pub struct CameraConfig {
    // Whether the main camera stays out of the ground.
    pub collision: bool,
    pub debug: DebugCameraConfig,
}

impl ConfigSection for CameraConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("collision", &mut self.collision);
        visitor.section("debug", &mut self.debug);
    }
}
