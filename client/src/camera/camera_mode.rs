#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    Orbit,
    Debug,
}

impl CameraMode {
    pub fn name(self) -> &'static str {
        match self {
            CameraMode::Orbit => "orbit",
            CameraMode::Debug => "debug",
        }
    }
}
