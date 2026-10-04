use glam::DVec3;

use crate::orbit::OrbitalElements;

#[derive(Clone, Copy)]
pub(super) struct Motion {
    pub elements: OrbitalElements,
    pub gravitational_parameter: f64,
}

impl Motion {
    pub fn position(&self, seconds_since_epoch: f64) -> DVec3 {
        self.elements
            .position(self.gravitational_parameter, seconds_since_epoch)
    }
}
