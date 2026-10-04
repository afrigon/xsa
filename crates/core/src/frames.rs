use std::f64::consts::FRAC_PI_2;

use glam::{DMat3, DQuat};

const EARTH_OBLIQUITY_DEGREES: f64 = 23.439_281;

pub fn equatorial_from_ecliptic() -> DMat3 {
    DMat3::from_rotation_x(EARTH_OBLIQUITY_DEGREES.to_radians())
}

// The plane's +X axis is its ascending node on the celestial equator, the origin JPL measures satellite nodes from.
pub fn plane_from_equatorial_pole(right_ascension: f64, declination: f64) -> DQuat {
    let ecliptic_from_equatorial = DQuat::from_mat3(&equatorial_from_ecliptic().transpose());
    ecliptic_from_equatorial
        * DQuat::from_rotation_z(right_ascension + FRAC_PI_2)
        * DQuat::from_rotation_x(FRAC_PI_2 - declination)
}

#[cfg(test)]
mod tests {
    use glam::DVec3;

    use super::*;

    fn equatorial_direction(right_ascension_hours: f64, declination_degrees: f64) -> DVec3 {
        let right_ascension = (right_ascension_hours * 15.0).to_radians();
        let declination = declination_degrees.to_radians();
        DVec3::new(
            declination.cos() * right_ascension.cos(),
            declination.cos() * right_ascension.sin(),
            declination.sin(),
        )
    }

    fn world_direction(right_ascension_hours: f64, declination_degrees: f64) -> DVec3 {
        equatorial_from_ecliptic().transpose() * equatorial_direction(right_ascension_hours, declination_degrees)
    }

    fn assert_directions_match(actual: DVec3, expected: DVec3) {
        let error_degrees = actual.angle_between(expected).to_degrees();
        assert!(
            error_degrees < 1e-6,
            "{actual} is {error_degrees}° away from {expected}"
        );
    }

    #[test]
    fn vernal_equinox_is_the_world_x_axis() {
        assert_directions_match(world_direction(0.0, 0.0), DVec3::X);
    }

    #[test]
    fn ecliptic_north_pole_is_the_world_z_axis() {
        assert_directions_match(world_direction(18.0, 90.0 - EARTH_OBLIQUITY_DEGREES), DVec3::Z);
    }

    #[test]
    fn summer_solstice_is_the_world_y_axis() {
        assert_directions_match(world_direction(6.0, EARTH_OBLIQUITY_DEGREES), DVec3::Y);
    }

    #[test]
    fn plane_normal_points_at_its_pole() {
        let plane = plane_from_equatorial_pole((268.0_f64).to_radians(), 64.5_f64.to_radians());
        assert_directions_match(plane * DVec3::Z, world_direction(268.0 / 15.0, 64.5));
    }

    #[test]
    fn plane_with_the_ecliptic_pole_is_the_ecliptic() {
        let plane = plane_from_equatorial_pole((270.0_f64).to_radians(), (90.0 - EARTH_OBLIQUITY_DEGREES).to_radians());
        assert_directions_match(plane * DVec3::Z, DVec3::Z);
        assert_directions_match(plane * DVec3::X, DVec3::X);
    }
}
