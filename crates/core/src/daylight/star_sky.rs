use xsa_units::SimulationTime;

use crate::simulation::{BodyIndex, Simulation, SimulationState};

// Where a star stands in a body's sky at a time, in the body's frame.
#[derive(Clone, Copy, Debug)]
pub struct StarSky {
    // Radians north of the body's equator.
    pub declination: f64,
    // Radians east of the prime meridian where the star is overhead.
    pub substellar_longitude: f64,
    // Radians from the star's centre to its edge.
    pub angular_radius: f64,
}

impl StarSky {
    pub fn at(simulation: &Simulation, body: BodyIndex, star: BodyIndex, time: SimulationTime) -> StarSky {
        let mut state = SimulationState::default();
        simulation.state_at(time, &mut state);
        let body_state = state.body(body);
        let offset = state.body(star).position - body_state.position;
        let distance = offset.length();
        let direction = body_state.orientation.inverse() * (offset / distance);

        StarSky {
            declination: direction.z.clamp(-1.0, 1.0).asin(),
            substellar_longitude: direction.y.atan2(direction.x),
            angular_radius: (simulation.body(star).radius / distance).min(1.0).asin(),
        }
    }
}
