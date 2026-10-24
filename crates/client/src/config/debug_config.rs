use xsa_config::{ConfigSection, ConfigVisitor};

use super::DebugCameraConfig;

#[derive(Clone, Debug, PartialEq)]
pub struct DebugConfig {
    pub wireframe: bool,
    pub camera: DebugCameraConfig,
}

impl ConfigSection for DebugConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("wireframe", &mut self.wireframe);
        visitor.section("camera", &mut self.camera);
    }
}
