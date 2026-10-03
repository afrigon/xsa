use glam::{DQuat, DVec3};

#[derive(Clone, Copy)]
pub struct BodyState {
    pub position: DVec3,
    pub orientation: DQuat,
}
