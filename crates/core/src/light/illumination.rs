use glam::DVec3;

use super::{BACKGROUND_ILLUMINANCE, StarLight};
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

    // Lux on a surface facing each star, before any shadowing by bodies.
    pub fn illuminance_at(&self, bodies: &[Body], states: &[BodyState], point: DVec3) -> f64 {
        let starlight: f64 = self
            .star_lights
            .iter()
            .zip(bodies)
            .zip(states)
            .filter_map(|((light, body), state)| {
                let light = light.as_ref()?;
                // Inside a star, the light is that of its surface.
                let distance = state.position.distance(point).max(body.radius);
                Some(light.luminous_intensity / (distance * distance))
            })
            .sum();

        BACKGROUND_ILLUMINANCE + starlight
    }
}

#[cfg(test)]
mod tests {
    use glam::DQuat;

    use super::*;
    use crate::random::Seed;
    use crate::simulation::{BodyCategory, BodyId, Star};

    const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;
    const SUN: Star = Star {
        luminosity: 3.828e26,
        effective_temperature: 5772.0,
    };
    const SUN_RADIUS: f64 = 6.957e8;
    const EARTH_RADIUS: f64 = 6.371e6;
    const LOW_ORBIT_ALTITUDE: f64 = 400e3;

    #[derive(Default)]
    struct System {
        bodies: Vec<Body>,
        states: Vec<BodyState>,
    }

    impl System {
        fn sun_and_earth() -> System {
            System::default().with("sol", SUN_RADIUS, Some(SUN), DVec3::ZERO).with(
                "earth",
                EARTH_RADIUS,
                None,
                DVec3::X * ASTRONOMICAL_UNIT,
            )
        }

        fn with(mut self, id: &str, radius: f64, star: Option<Star>, position: DVec3) -> System {
            self.bodies.push(Body {
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
                seed: Seed::from_id(id),
            });
            self.states.push(BodyState {
                position,
                orientation: DQuat::IDENTITY,
            });
            self
        }

        fn illuminance_at(&self, point: DVec3) -> f64 {
            Illumination::new(&self.bodies).illuminance_at(&self.bodies, &self.states, point)
        }
    }

    #[test]
    fn sunlight_beside_the_earth_is_about_128_kilolux() {
        let illuminance =
            System::sun_and_earth().illuminance_at(DVec3::new(ASTRONOMICAL_UNIT, EARTH_RADIUS * 2.0, 0.0));
        assert!((illuminance - 128_000.0).abs() < 5_000.0, "{illuminance} lux");
    }

    #[test]
    fn a_body_in_between_changes_nothing() {
        let system = System::sun_and_earth();
        let behind = system.illuminance_at(DVec3::X * (ASTRONOMICAL_UNIT + EARTH_RADIUS + LOW_ORBIT_ALTITUDE));
        let beside = system.illuminance_at(DVec3::new(ASTRONOMICAL_UNIT, EARTH_RADIUS + LOW_ORBIT_ALTITUDE, 0.0));
        assert!(
            (behind - beside).abs() / beside < 1e-4,
            "{behind} lux behind, {beside} lux beside"
        );
    }

    #[test]
    fn the_light_of_every_star_adds_up() {
        let point = DVec3::X * ASTRONOMICAL_UNIT;
        let one = System::default().with("sol", SUN_RADIUS, Some(SUN), DVec3::ZERO);
        let two = System::default().with("sol", SUN_RADIUS, Some(SUN), DVec3::ZERO).with(
            "twin",
            SUN_RADIUS,
            Some(SUN),
            DVec3::X * 2.0 * ASTRONOMICAL_UNIT,
        );
        let single = one.illuminance_at(point) - BACKGROUND_ILLUMINANCE;
        let double = two.illuminance_at(point) - BACKGROUND_ILLUMINANCE;
        assert!(
            (double - 2.0 * single).abs() / single < 1e-9,
            "{single} lux, {double} lux"
        );
    }

    #[test]
    fn inside_the_star_the_light_is_its_surface_light() {
        let system = System::sun_and_earth();
        let surface = Illumination::new(&system.bodies)
            .star_light(BodyIndex { value: 0 })
            .unwrap()
            .surface_luminance;
        let inside = system.illuminance_at(DVec3::X * SUN_RADIUS * 0.5) - BACKGROUND_ILLUMINANCE;
        let expected = std::f64::consts::PI * surface;
        assert!((inside - expected).abs() / expected < 1e-6, "{inside} lux");
    }
}
