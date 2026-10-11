use glam::DVec3;

use super::{BACKGROUND_ILLUMINANCE, StarLight, visible_disk_fraction};
use crate::simulation::{Body, BodyIndex, BodyState};

pub struct Illumination {
    star_lights: Vec<Option<StarLight>>,
}

impl Illumination {
    pub fn new(bodies: &[Body]) -> Illumination {
        Illumination {
            star_lights: bodies
                .iter()
                .map(|body| body.star.map(|star| StarLight::of_star(&star, body.radius)))
                .collect(),
        }
    }

    pub fn star_light(&self, index: BodyIndex) -> Option<&StarLight> {
        self.star_lights[index.value].as_ref()
    }

    // Lux on a surface facing each star.
    pub fn illuminance_at(&self, bodies: &[Body], states: &[BodyState], point: DVec3) -> f64 {
        let starlight: f64 = self
            .star_lights
            .iter()
            .enumerate()
            .filter_map(|(value, light)| light.as_ref().map(|light| (BodyIndex { value }, light)))
            .map(|(star, light)| {
                let radius = bodies[star.value].radius;
                let distance = states[star.value].position.distance(point);

                if distance <= radius {
                    return light.luminous_intensity / (radius * radius);
                }

                light.luminous_intensity / (distance * distance)
                    * Illumination::visible_fraction(bodies, states, star, point)
            })
            .sum();

        BACKGROUND_ILLUMINANCE + starlight
    }

    fn visible_fraction(bodies: &[Body], states: &[BodyState], star: BodyIndex, point: DVec3) -> f64 {
        let star_offset = states[star.value].position - point;
        let star_distance = star_offset.length();
        let star_radius = (bodies[star.value].radius / star_distance).asin();

        bodies
            .iter()
            .zip(states)
            .enumerate()
            .filter(|&(value, _)| value != star.value)
            .map(|(_, (body, state))| {
                let offset = state.position - point;
                let distance = offset.length();

                if distance <= body.radius {
                    return 0.0;
                }

                if distance >= star_distance {
                    return 1.0;
                }

                visible_disk_fraction(
                    star_radius,
                    (body.radius / distance).asin(),
                    offset.angle_between(star_offset),
                )
            })
            .product()
    }
}

#[cfg(test)]
mod tests {
    use glam::DQuat;

    use super::*;
    use crate::simulation::{BodyCategory, BodyId, Star};

    const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;
    const SUN_RADIUS: f64 = 6.957e8;
    const EARTH_RADIUS: f64 = 6.371e6;
    const LOW_ORBIT_ALTITUDE: f64 = 400e3;

    fn body(id: &str, radius: f64, star: Option<Star>) -> Body {
        Body {
            id: BodyId { value: id.to_string() },
            category: if star.is_some() {
                BodyCategory::Star
            } else {
                BodyCategory::Planet
            },
            parent: None,
            radius,
            gravitational_parameter: 0.0,
            star,
        }
    }

    fn state(position: DVec3) -> BodyState {
        BodyState {
            position,
            orientation: DQuat::IDENTITY,
        }
    }

    fn sun_and_earth() -> (Vec<Body>, Vec<BodyState>) {
        let sun = Star {
            luminosity: 3.828e26,
            effective_temperature: 5772.0,
        };
        (
            vec![body("sol", SUN_RADIUS, Some(sun)), body("earth", EARTH_RADIUS, None)],
            vec![state(DVec3::ZERO), state(DVec3::X * ASTRONOMICAL_UNIT)],
        )
    }

    fn illuminance_at(point: DVec3) -> f64 {
        let (bodies, states) = sun_and_earth();
        Illumination::new(&bodies).illuminance_at(&bodies, &states, point)
    }

    #[test]
    fn sunlight_beside_the_earth_is_about_128_kilolux() {
        let illuminance = illuminance_at(DVec3::new(ASTRONOMICAL_UNIT, EARTH_RADIUS * 2.0, 0.0));
        assert!((illuminance - 128_000.0).abs() < 5_000.0, "{illuminance} lux");
    }

    #[test]
    fn the_earth_shadow_leaves_only_the_background() {
        let point = DVec3::X * (ASTRONOMICAL_UNIT + EARTH_RADIUS + LOW_ORBIT_ALTITUDE);
        assert_eq!(illuminance_at(point), BACKGROUND_ILLUMINANCE);
    }

    #[test]
    fn a_point_inside_a_body_is_dark() {
        assert_eq!(illuminance_at(DVec3::X * ASTRONOMICAL_UNIT), BACKGROUND_ILLUMINANCE);
    }

    #[test]
    fn inside_the_star_the_light_is_its_surface_light() {
        let (bodies, _) = sun_and_earth();
        let illumination = Illumination::new(&bodies);
        let surface = illumination
            .star_light(BodyIndex { value: 0 })
            .unwrap()
            .surface_luminance;
        let inside = illuminance_at(DVec3::X * SUN_RADIUS * 0.5) - BACKGROUND_ILLUMINANCE;
        let expected = std::f64::consts::PI * surface;
        assert!((inside - expected).abs() / expected < 1e-6, "{inside} lux");
    }

    #[test]
    fn the_penumbra_dims_the_light_gradually() {
        let behind = ASTRONOMICAL_UNIT + EARTH_RADIUS + LOW_ORBIT_ALTITUDE;
        let samples: Vec<f64> = (0..=40)
            .map(|step| illuminance_at(DVec3::new(behind, EARTH_RADIUS * (0.99 + step as f64 * 0.0005), 0.0)))
            .collect();
        assert!(
            samples.windows(2).all(|pair| pair[1] >= pair[0] * (1.0 - 1e-6)),
            "{samples:?}"
        );
        assert!(
            samples.iter().any(|&lux| lux > 1_000.0 && lux < 100_000.0),
            "{samples:?}"
        );
    }
}
