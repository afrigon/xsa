use std::path::PathBuf;

use xsa_core::simulation::{BodyId, Simulation, SimulationState};
use xsa_packs::{Id, PackStack, SimulationDefinition};
use xsa_units::SimulationTime;

const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;

struct Positions {
    sun: glam::DVec3,
    earth: glam::DVec3,
    moon: glam::DVec3,
}

fn load() -> Simulation {
    let packs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let stack = PackStack::load(&packs, &["base".to_string(), "system-solar".to_string()]).unwrap();
    let id = Id::parse("system-solar:sol", "base").unwrap();
    let definition = stack.load_data::<SimulationDefinition>(&id).unwrap();
    definition.build(&stack).unwrap()
}

fn positions_at(simulation: &Simulation, timestamp: &str) -> Positions {
    let mut state = SimulationState::default();
    simulation.state_at(timestamp.parse::<SimulationTime>().unwrap(), &mut state);
    let position = |id: &str| {
        let id = BodyId {
            value: format!("system-solar:{id}"),
        };
        state.body(simulation.find_body(&id).unwrap()).position
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

struct Reference {
    body: &'static str,
    parent: &'static str,
    kilometers: [f64; 3],
    tolerance_degrees: f64,
}

// JPL Horizons, ecliptic J2000, relative to the parent's center at 2026-10-01T00:00 TDB.
const HORIZONS_2026_10_01: [Reference; 11] = [
    Reference {
        body: "jupiter",
        parent: "sol",
        kilometers: [-5.221632141e8, 5.979565007e8, 9.198782169e6],
        tolerance_degrees: 0.05,
    },
    Reference {
        body: "saturn",
        parent: "sol",
        kilometers: [1.385149691e9, 2.642948909e8, -5.974019060e7],
        tolerance_degrees: 0.2,
    },
    Reference {
        body: "pluto",
        parent: "sol",
        kilometers: [2.986604697e9, -4.393689164e9, -3.936314894e8],
        tolerance_degrees: 0.05,
    },
    Reference {
        body: "io",
        parent: "jupiter",
        kilometers: [4.405540914e4, 4.183189810e5, 1.575805527e4],
        tolerance_degrees: 0.05,
    },
    Reference {
        body: "ganymede",
        parent: "jupiter",
        kilometers: [8.405344405e5, 6.585803899e5, 3.745925331e4],
        tolerance_degrees: 0.15,
    },
    Reference {
        body: "titan",
        parent: "saturn",
        kilometers: [1.068315076e6, 4.223177262e5, -3.240312691e5],
        tolerance_degrees: 0.05,
    },
    Reference {
        body: "iapetus",
        parent: "saturn",
        kilometers: [-6.609833738e4, -3.487491832e6, 8.151935736e5],
        tolerance_degrees: 0.1,
    },
    Reference {
        body: "titania",
        parent: "uranus",
        kilometers: [-4.027762100e5, 6.697867458e4, -1.523246248e5],
        tolerance_degrees: 0.15,
    },
    Reference {
        body: "triton",
        parent: "neptune",
        kilometers: [3.010274569e5, 1.127751974e5, -1.500855254e5],
        tolerance_degrees: 0.05,
    },
    Reference {
        body: "phobos",
        parent: "mars",
        kilometers: [-8.297237210e2, -9.343751083e3, -2.010869397e2],
        tolerance_degrees: 0.5,
    },
    Reference {
        body: "hydra",
        parent: "pluto",
        kilometers: [3.182607067e4, 5.183839361e4, 2.776632149e4],
        tolerance_degrees: 2.5,
    },
];

#[test]
fn bodies_match_horizons_on_2026_10_01() {
    let simulation = load();
    let mut state = SimulationState::default();
    simulation.state_at("2026-10-01T00:00:00Z".parse::<SimulationTime>().unwrap(), &mut state);
    let position = |id: &str| {
        let id = BodyId {
            value: format!("system-solar:{id}"),
        };
        state.body(simulation.find_body(&id).unwrap()).position
    };

    for reference in &HORIZONS_2026_10_01 {
        let actual = position(reference.body) - position(reference.parent);
        let expected = glam::DVec3::from_array(reference.kilometers) * 1000.0;
        let error_degrees = actual.angle_between(expected).to_degrees();
        let distance_error = (actual.length() - expected.length()).abs() / expected.length();
        assert!(
            error_degrees < reference.tolerance_degrees && distance_error < 0.01,
            "{}: {error_degrees}° and {distance_error} relative distance away from Horizons",
            reference.body
        );
    }
}

fn body_id(path: &str) -> BodyId {
    BodyId {
        value: format!("system-solar:{path}"),
    }
}

fn parent_path(simulation: &Simulation, path: &str) -> Option<String> {
    let body = simulation.body(simulation.find_body(&body_id(path)).unwrap());

    body.parent.map(|parent| simulation.body(parent).id.value.clone())
}

struct ParentCase {
    body: &'static str,
    parent: &'static str,
}

#[test]
fn bodies_know_the_body_they_orbit() {
    let simulation = load();
    assert_eq!(parent_path(&simulation, "sol"), None);
    let cases = [
        ParentCase {
            body: "earth",
            parent: "sol",
        },
        ParentCase {
            body: "luna",
            parent: "earth",
        },
        ParentCase {
            body: "phobos",
            parent: "mars",
        },
        ParentCase {
            body: "charon",
            parent: "pluto",
        },
        ParentCase {
            body: "nix",
            parent: "pluto",
        },
        ParentCase {
            body: "dactyl",
            parent: "ida",
        },
    ];

    for case in cases {
        let expected = format!("system-solar:{}", case.parent);
        assert_eq!(parent_path(&simulation, case.body), Some(expected), "{}", case.body);
    }
}
