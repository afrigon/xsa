use glam::{DVec2, DVec3};

const SAMPLE_COUNT: usize = 256;
// The golden angle spreads consecutive samples of a Vogel spiral evenly around the disk.
const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;

// Measures how much of a star's disk the bodies in front of it hide, by casting a fixed pattern of rays from the
// camera across the disk: the pattern never moves, so the result is stable from frame to frame.
pub(in crate::renderer) struct StarOcclusion {
    // Evenly spread over the unit disk.
    samples: Vec<DVec2>,
}

impl StarOcclusion {
    pub fn new() -> StarOcclusion {
        let samples = (0..SAMPLE_COUNT)
            .map(|index| {
                let radius = ((index as f64 + 0.5) / SAMPLE_COUNT as f64).sqrt();
                let angle = index as f64 * GOLDEN_ANGLE;
                DVec2::new(angle.cos(), angle.sin()) * radius
            })
            .collect();

        StarOcclusion { samples }
    }

    // Positions are relative to the camera; occluders are spheres given as (center, radius).
    pub fn visibility(&self, star: DVec3, star_radius: f64, occluders: impl Iterator<Item = (DVec3, f64)>) -> f32 {
        let star_distance = star.length();
        let direction = star / star_distance;
        let star_angular_radius = (star_radius / star_distance).asin();
        let in_front: Vec<(DVec3, f64)> = occluders
            .filter(|&(center, radius)| {
                StarOcclusion::may_cover(direction, star_distance, star_angular_radius, center, radius)
            })
            .collect();

        if in_front.is_empty() {
            return 1.0;
        }

        if in_front.iter().any(|&(center, radius)| center.length() <= radius) {
            return 0.0;
        }

        let (tangent, bitangent) = direction.any_orthonormal_pair();
        let spread = star_angular_radius.tan();
        let visible = self
            .samples
            .iter()
            .filter(|sample| {
                let ray = (direction + (tangent * sample.x + bitangent * sample.y) * spread).normalize();

                !in_front
                    .iter()
                    .any(|&(center, radius)| StarOcclusion::hits(ray, star_distance, center, radius))
            })
            .count();

        visible as f32 / SAMPLE_COUNT as f32
    }

    fn may_cover(direction: DVec3, star_distance: f64, star_angular_radius: f64, center: DVec3, radius: f64) -> bool {
        let distance = center.length();

        if distance <= radius {
            return true;
        }

        if distance - radius >= star_distance {
            return false;
        }

        let separation = direction.angle_between(center / distance);

        separation < star_angular_radius + (radius / distance).asin()
    }

    // Whether a ray from the camera meets the sphere before reaching the star.
    fn hits(ray: DVec3, star_distance: f64, center: DVec3, radius: f64) -> bool {
        let closest_approach = ray.dot(center);

        if closest_approach <= 0.0 {
            return false;
        }

        let miss_squared = center.length_squared() - closest_approach * closest_approach;
        let radius_squared = radius * radius;

        miss_squared < radius_squared && closest_approach - (radius_squared - miss_squared).sqrt() < star_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUN_RADIUS: f64 = 6.957e8;
    const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;
    const EARTH_RADIUS: f64 = 6.371e6;

    fn sun() -> DVec3 {
        DVec3::new(0.0, ASTRONOMICAL_UNIT, 0.0)
    }

    // An Earth-sized sphere seen from 25 000 km, whose limb sits `offset` star radii from the star's center.
    fn earth_with_limb_at(offset: f64) -> (DVec3, f64) {
        let distance = 2.5e7;
        let limb_angle = (EARTH_RADIUS / distance).asin();
        let star_angular_radius = (SUN_RADIUS / ASTRONOMICAL_UNIT).asin();
        let angle = limb_angle + offset * star_angular_radius;

        (DVec3::new(angle.sin(), angle.cos(), 0.0) * distance, EARTH_RADIUS)
    }

    #[test]
    fn an_unobstructed_star_is_fully_visible() {
        let visibility = StarOcclusion::new().visibility(sun(), SUN_RADIUS, std::iter::once(earth_with_limb_at(2.0)));

        assert_eq!(visibility, 1.0);
    }

    #[test]
    fn a_star_behind_a_body_is_hidden() {
        let visibility = StarOcclusion::new().visibility(sun(), SUN_RADIUS, std::iter::once(earth_with_limb_at(-2.0)));

        assert_eq!(visibility, 0.0);
    }

    #[test]
    fn a_limb_through_the_center_hides_half_the_disk() {
        let visibility = StarOcclusion::new().visibility(sun(), SUN_RADIUS, std::iter::once(earth_with_limb_at(0.0)));

        assert!((visibility - 0.5).abs() < 0.03, "{visibility}");
    }

    #[test]
    fn a_body_behind_the_star_hides_nothing() {
        let behind = (sun() * 2.0, EARTH_RADIUS * 1000.0);

        assert_eq!(
            StarOcclusion::new().visibility(sun(), SUN_RADIUS, std::iter::once(behind)),
            1.0
        );
    }
}
