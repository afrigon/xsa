use xui::EnvironmentKey;

// The name of the body the camera orbits, for the game's HUD.
pub struct TargetNameKey;

impl EnvironmentKey for TargetNameKey {
    type Value = Option<String>;

    fn default_value() -> Option<String> {
        None
    }
}
