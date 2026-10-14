use glam::{DMat3, DQuat, DVec3, EulerRot};

// Radians against a horizon given by its up and north: yaw counterclockwise around up from north, roll clockwise
// around the view direction, pitch around the rolled right (above the horizon when level).
#[derive(Clone, Copy, Debug)]
pub struct Attitude {
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
}

impl Attitude {
    // `up` and `north` are perpendicular unit vectors; the camera looks along its +Y with +X to its right.
    pub fn of(orientation: DQuat, up: DVec3, north: DVec3) -> Attitude {
        let horizon = DQuat::from_mat3(&DMat3::from_cols(north.cross(up), north, up));
        // Yaw, then roll around the view, then pitch around the rolled right: only a view whose right points straight
        // up or down is ambiguous, which a camera without roll never reaches, even looking straight down.
        let (yaw, roll, pitch) = (horizon.inverse() * orientation).to_euler(EulerRot::ZYX);

        Attitude { yaw, pitch, roll }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use super::*;

    const TOLERANCE: f64 = 1e-12;

    fn attitude(orientation: DQuat) -> Attitude {
        Attitude::of(orientation, DVec3::Z, DVec3::Y)
    }

    #[test]
    fn looking_north_is_level() {
        let level = attitude(DQuat::IDENTITY);

        assert!(level.yaw.abs() < TOLERANCE && level.pitch.abs() < TOLERANCE && level.roll.abs() < TOLERANCE);
    }

    #[test]
    fn yaw_and_pitch_match_the_orbit_camera() {
        let yaw = 0.7;
        let pitch = -0.3;
        let reading = attitude(DQuat::from_rotation_z(yaw) * DQuat::from_rotation_x(pitch));

        assert!((reading.yaw - yaw).abs() < TOLERANCE);
        assert!((reading.pitch - pitch).abs() < TOLERANCE);
        assert!(reading.roll.abs() < TOLERANCE);
    }

    #[test]
    fn roll_turns_clockwise_around_the_view() {
        let roll = 0.4;
        let reading = attitude(DQuat::from_rotation_y(roll));

        assert!((reading.roll - roll).abs() < TOLERANCE);
    }

    #[test]
    fn looking_straight_up_keeps_a_heading() {
        let reading = attitude(DQuat::from_rotation_x(FRAC_PI_2));

        assert!((reading.pitch - FRAC_PI_2).abs() < TOLERANCE);
        assert!(reading.roll.is_finite() && reading.yaw.is_finite());
    }

    #[test]
    fn looking_straight_down_keeps_its_yaw() {
        let yaw = 0.8;
        let reading = attitude(DQuat::from_rotation_z(yaw) * DQuat::from_rotation_x(-FRAC_PI_2));

        assert!((reading.yaw - yaw).abs() < TOLERANCE);
        assert!((reading.pitch + FRAC_PI_2).abs() < TOLERANCE);
        assert!(reading.roll.abs() < TOLERANCE);
    }
}
