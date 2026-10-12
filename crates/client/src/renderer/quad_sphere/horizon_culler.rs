use std::f64::consts::PI;

use glam::DVec3;

// Seen from outside a sphere, a point on its surface is visible exactly when its angle from the camera's direction,
// at the sphere's center, is under the horizon angle acos(radius / distance).
pub(in crate::renderer) struct HorizonCuller {
    camera_direction: DVec3,
    horizon_angle: f64,
}

impl HorizonCuller {
    // `camera` is relative to the sphere's center.
    pub fn new(camera: DVec3, radius: f64) -> Self {
        let distance = camera.length();
        let horizon_angle = if distance > radius {
            (radius / distance).acos()
        } else {
            PI
        };

        Self {
            camera_direction: camera / distance,
            horizon_angle,
        }
    }

    // Whether a cap of the surface, centered on `direction`, lies entirely past the horizon.
    pub fn is_hidden(&self, direction: DVec3, angular_radius: f64) -> bool {
        let angle = self
            .camera_direction
            .cross(direction)
            .length()
            .atan2(self.camera_direction.dot(direction));

        angle - angular_radius > self.horizon_angle
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
        assert!(!culler().is_hidden(direction_at(0.0), SMALL_CAP));
        assert!(!culler().is_hidden(direction_at(55.0), SMALL_CAP));
    }

    #[test]
    fn the_far_side_is_hidden() {
        assert!(culler().is_hidden(direction_at(180.0), SMALL_CAP));
        assert!(culler().is_hidden(direction_at(65.0), SMALL_CAP));
    }

    #[test]
    fn a_cap_straddling_the_horizon_is_visible() {
        assert!(!culler().is_hidden(direction_at(70.0), 15f64.to_radians()));
    }

    #[test]
    fn nothing_is_hidden_from_inside_the_sphere() {
        let inside = HorizonCuller::new(DVec3::new(0.0, 0.0, 0.5), RADIUS);

        assert!(!inside.is_hidden(direction_at(180.0), SMALL_CAP));
    }

    #[test]
    fn the_horizon_closes_in_near_the_surface() {
        let altitude = 1.0e-6;
        let low = HorizonCuller::new(DVec3::new(0.0, 0.0, RADIUS + altitude), RADIUS);
        let horizon = (2.0 * altitude).sqrt();

        assert!(!low.is_hidden(DVec3::new((horizon * 0.9).sin(), 0.0, (horizon * 0.9).cos()), 0.0));
        assert!(low.is_hidden(DVec3::new((horizon * 1.1).sin(), 0.0, (horizon * 1.1).cos()), 0.0));
    }
}
