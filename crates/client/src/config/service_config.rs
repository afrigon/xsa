use xsa_config::{ConfigSection, ConfigVisitor};

use super::ResourcesConfig;

#[derive(Clone, Debug, PartialEq)]
pub struct ServiceConfig {
    pub resources: ResourcesConfig,
}

impl ConfigSection for ServiceConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.section("resources", &mut self.resources);
    }
}
