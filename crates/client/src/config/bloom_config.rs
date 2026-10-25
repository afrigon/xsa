use xsa_config::{ConfigSection, ConfigVisitor, NumberRange};

#[derive(Clone, Debug, PartialEq)]
pub struct BloomConfig {
    pub enabled: bool,
    pub strength: f32,
}

impl ConfigSection for BloomConfig {
    fn visit<V: ConfigVisitor>(&mut self, visitor: &mut V) {
        visitor.value("enabled", &mut self.enabled);
        visitor.number("strength", &mut self.strength, NumberRange::at_least(0.0));
    }
}
