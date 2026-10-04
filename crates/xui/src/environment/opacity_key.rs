use super::EnvironmentKey;

// The opacity everything inside is drawn with: `.opacity()` multiplies it, so nested opacities combine.
pub struct OpacityKey;

impl EnvironmentKey for OpacityKey {
    type Value = f32;

    fn default_value() -> f32 {
        1.0
    }
}
