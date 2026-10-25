use glam::{DQuat, DVec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPose {
    pub position: DVec3,
    pub orientation: DQuat,
}
