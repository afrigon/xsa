#[derive(Clone, Debug, PartialEq)]
pub struct AdaptationConfig {
    pub dark_to_light_half_life_seconds: f32,
    pub light_to_dark_half_life_seconds: f32,
}
