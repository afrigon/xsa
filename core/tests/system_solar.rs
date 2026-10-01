use std::path::PathBuf;

use xsa_core::packs::{Id, PackStack};
use xsa_core::simulation::{Simulation, SimulationState};
use xsa_core::time;

const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;

struct Positions {
    sun: glam::DVec3,
    earth: glam::DVec3,
    moon: glam::DVec3,
}

fn load() -> Simulation {
    let packs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../packs");
    let stack = PackStack::load(&packs, &["base".to_string(), "system-solar".to_string()]).unwrap();
    Simulation::load(&stack, &Id::parse("system-solar:sol", "base").unwrap()).unwrap()
}

fn positions_at(simulation: &Simulation, timestamp: &str) -> Positions {
    let mut state = SimulationState::default();
    simulation.state_at(time::parse_timestamp(timestamp).unwrap(), &mut state);
    let position = |id: &str| {
        let index = simulation.bodies().iter().position(|body| body.id.path == id).unwrap();
        state.bodies[index].position
    };
    Positions {
        sun: position("sol"),
        earth: position("earth"),
        moon: position("luna"),
    }
}

fn sun_moon_angle_degrees(positions: &Positions) -> f64 {
    (positions.sun - positions.earth)
        .angle_between(positions.moon - positions.earth)
        .to_degrees()
}

#[test]
fn earth_reaches_perihelion_in_early_january_2026() {
    let positions = positions_at(&load(), "2026-01-03T17:00:00Z");
    let distance = positions.earth.distance(positions.sun) / ASTRONOMICAL_UNIT;
    assert!((distance - 0.98330).abs() < 0.0003, "Earth–Sun distance {distance} AU");
}

#[test]
fn moon_is_opposite_the_sun_at_the_january_2026_full_moon() {
    let angle = sun_moon_angle_degrees(&positions_at(&load(), "2026-01-03T10:03:00Z"));
    assert!(angle > 175.0, "Sun–Moon angle {angle}°");
}

#[test]
fn moon_is_beside_the_sun_at_the_january_2026_new_moon() {
    let angle = sun_moon_angle_degrees(&positions_at(&load(), "2026-01-18T19:52:00Z"));
    assert!(angle < 5.0, "Sun–Moon angle {angle}°");
}

#[test]
fn moon_stays_within_its_real_distance_range() {
    let simulation = load();
    for day in 1..=28 {
        let positions = positions_at(&simulation, &format!("2026-02-{day:02}T00:00:00Z"));
        let distance = positions.moon.distance(positions.earth) / 1000.0;
        assert!((356_000.0..407_000.0).contains(&distance), "day {day}: {distance} km");
    }
}
