#[derive(Clone, Debug, PartialEq)]
pub struct BloomConfig {
    pub enabled: bool,
    pub strength: f32,
}

impl BloomConfig {
    pub fn effective_strength(&self) -> f32 {
        if self.enabled { self.strength } else { 0.0 }
    }
}
