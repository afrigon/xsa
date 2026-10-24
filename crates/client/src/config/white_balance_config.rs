use xsa_config::{ConfigSection, ConfigVisitor};

#[derive(Clone, Debug, PartialEq)]
pub struct WhiteBalanceConfig {
    pub enabled: bool,
}

impl ConfigSection for WhiteBalanceConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
    }
}
