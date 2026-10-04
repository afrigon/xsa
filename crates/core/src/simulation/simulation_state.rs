use glam::DVec3;

use super::{BodyIndex, BodyState};

#[derive(Default)]
pub struct SimulationState {
    pub(super) positions: Vec<DVec3>,
    pub bodies: Vec<BodyState>,
}

impl SimulationState {
    pub fn body(&self, index: BodyIndex) -> &BodyState {
        &self.bodies[index.value]
    }
}
