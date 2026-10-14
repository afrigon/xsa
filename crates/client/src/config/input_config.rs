#[derive(Clone, Debug, PartialEq)]
pub struct InputConfig {
    // Off, the camera ignores the mouse and keyboard and never captures the mouse; key binds still run.
    pub enabled: bool,
}
