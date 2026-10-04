use super::EnvironmentKey;

// Physical pixels per logical point.
pub struct ScaleFactorKey;

impl EnvironmentKey for ScaleFactorKey {
    type Value = f32;

    fn default_value() -> f32 {
        1.0
    }
}
