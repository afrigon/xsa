use std::f64::consts::{PI, TAU};

use glam::DQuat;

use super::Spin;
use super::motion::Motion;

pub(super) struct Rotation {
    pub reference: Option<Motion>,
    pub spin: Spin,
    pub axial_tilt: f64,
    pub period: f64,
}

impl Rotation {
    pub fn orientation(&self, seconds_since_epoch: f64) -> DQuat {
        let plane = self.reference.map_or(DQuat::IDENTITY, |reference| {
            reference.elements.plane_orientation(seconds_since_epoch)
        });
        let angle = match self.reference {
            Some(reference) if self.spin.tidally_locked => {
                reference.elements.periapsis_at(seconds_since_epoch)
                    + reference
                        .elements
                        .mean_anomaly_at(reference.gravitational_parameter, seconds_since_epoch)
                    + PI
                    + self.spin.prime_meridian
            }
            _ => self.spin.prime_meridian + TAU * seconds_since_epoch / self.period,
        };

        plane
            * DQuat::from_rotation_z(self.spin.azimuth)
            * DQuat::from_rotation_x(self.axial_tilt)
            * DQuat::from_rotation_z(angle)
    }
}
