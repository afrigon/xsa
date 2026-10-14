use std::f64::consts::PI;

use glam::DVec3;

// Seen from outside the sphere under the lowest terrain, a point on it is visible exactly when its angle from the
// camera's direction, at the center, is under acos(lowest / distance). A peak reaching `highest` rises above that
// horizon until acos(lowest / highest) further.
pub(in crate::renderer) struct HorizonCuller {
    camera_direction: DVec3,
    horizon_angle: f64,
    lowest_radius: f64,
}

impl HorizonCuller {
    // `camera` is relative to the sphere's center.
    pub fn new(camera: DVec3, lowest_radius: f64) -> Self {
        let distance = camera.length();
        let horizon_angle = if distance > lowest_radius {
            (lowest_radius / distance).acos()
        } else {
            PI
        };

        Self {
            camera_direction: camera / distance,
            horizon_angle,
            lowest_radius,
        }
    }

    // Whether a cap of the surface, centered on `direction` and reaching up to `highest_radius`, lies entirely past
    // the horizon.
    pub fn is_hidden(&self, direction: DVec3, angular_radius: f64, highest_radius: f64) -> bool {
        let angle = self
            .camera_direction
            .cross(direction)
            .length()
            .atan2(self.camera_direction.dot(direction));
        let rise = (self.lowest_radius / highest_radius).min(1.0).acos();

        angle - angular_radius > self.horizon_angle + rise
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RADIUS: f64 = 1.0;
    const SMALL_CAP: f64 = 0.01;

    // From twice the radius, the horizon is 60° from the camera's direction.
    fn culler() -> HorizonCuller {
        HorizonCuller::new(DVec3::new(0.0, 0.0, 2.0), RADIUS)
    }

    fn direction_at(angle_degrees: f64) -> DVec3 {
        let angle = angle_degrees.to_radians();
        DVec3::new(angle.sin(), 0.0, angle.cos())
    }

    #[test]
    fn the_near_side_is_visible() {
        assert!(!culler().is_hidden(direction_at(0.0), SMALL_CAP, RADIUS));
        assert!(!culler().is_hidden(direction_at(55.0), SMALL_CAP, RADIUS));
    }

    #[test]
    fn the_far_side_is_hidden() {
        assert!(culler().is_hidden(direction_at(180.0), SMALL_CAP, RADIUS));
        assert!(culler().is_hidden(direction_at(65.0), SMALL_CAP, RADIUS));
    }

    #[test]
    fn a_cap_straddling_the_horizon_is_visible() {
        assert!(!culler().is_hidden(direction_at(70.0), 15f64.to_radians(), RADIUS));
    }

    #[test]
    fn nothing_is_hidden_from_inside_the_sphere() {
        let inside = HorizonCuller::new(DVec3::new(0.0, 0.0, 0.5), RADIUS);

        assert!(!inside.is_hidden(direction_at(180.0), SMALL_CAP, RADIUS));
    }

    #[test]
    fn the_horizon_closes_in_near_the_surface() {
        let altitude = 1.0e-6;
        let low = HorizonCuller::new(DVec3::new(0.0, 0.0, RADIUS + altitude), RADIUS);
        let horizon = (2.0 * altitude).sqrt();

        assert!(!low.is_hidden(
            DVec3::new((horizon * 0.9).sin(), 0.0, (horizon * 0.9).cos()),
            0.0,
            RADIUS
        ));
        assert!(low.is_hidden(
            DVec3::new((horizon * 1.1).sin(), 0.0, (horizon * 1.1).cos()),
            0.0,
            RADIUS
        ));
    }

    // From twice the radius, a peak at twice the radius is seen up to 60° + 60° away.
    #[test]
    fn peaks_rise_above_the_horizon() {
        let mountainous = HorizonCuller::new(DVec3::new(0.0, 0.0, 2.0), RADIUS);

        assert!(!mountainous.is_hidden(direction_at(115.0), SMALL_CAP, 2.0 * RADIUS));
        assert!(mountainous.is_hidden(direction_at(125.0), SMALL_CAP, 2.0 * RADIUS));
    }
}
