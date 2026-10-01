use glam::{DQuat, DVec3};

const ASTRONOMICAL_UNIT: f64 = 149_597_870_700.0;
const EARTH_MOON_DISTANCE: f64 = 384_400_000.0;
const EARTH_ORBITAL_SPEED: f64 = 29_780.0;
const MOON_ORBITAL_SPEED: f64 = 1_022.0;
const EARTH_ROTATION_RATE: f64 = 7.292_115_9e-5;

pub struct Body {
    pub id: String,
    pub position: DVec3,
    pub velocity: DVec3,
    pub orientation: DQuat,
    pub angular_velocity: DVec3,
    pub radius: f64,
    pub gravitational_parameter: f64,
}

pub struct Simulation {
    pub bodies: Vec<Body>,
}

impl Simulation {
    pub fn solar_system() -> Self {
        let earth_position = DVec3::new(ASTRONOMICAL_UNIT, 0.0, 0.0);
        let earth_velocity = DVec3::new(0.0, EARTH_ORBITAL_SPEED, 0.0);
        let moon_direction = DVec3::new(0.6, 1.0, 0.0).normalize();
        let moon_prograde = DVec3::Z.cross(moon_direction);

        let bodies = vec![
            Body {
                id: "sol".into(),
                position: DVec3::ZERO,
                velocity: DVec3::ZERO,
                orientation: DQuat::IDENTITY,
                angular_velocity: DVec3::ZERO,
                radius: 695_700_000.0,
                gravitational_parameter: 1.327_124_400_18e20,
            },
            Body {
                id: "earth".into(),
                position: earth_position,
                velocity: earth_velocity,
                orientation: DQuat::IDENTITY,
                angular_velocity: DVec3::new(0.0, 0.0, EARTH_ROTATION_RATE),
                radius: 6_371_000.0,
                gravitational_parameter: 3.986_004_418e14,
            },
            Body {
                id: "luna".into(),
                position: earth_position + moon_direction * EARTH_MOON_DISTANCE,
                velocity: earth_velocity + moon_prograde * MOON_ORBITAL_SPEED,
                orientation: DQuat::IDENTITY,
                angular_velocity: DVec3::ZERO,
                radius: 1_737_400.0,
                gravitational_parameter: 4.904_869_5e12,
            },
        ];
        Self { bodies }
    }
}
