use xsa_units::SimulationTime;

use crate::simulation::{BodyIndex, Simulation, SimulationState};

// Where one body stands in another's sky at a time, in the frame of the body whose sky it is.
#[derive(Clone, Copy, Debug)]
pub struct SkyPosition {
    // Radians north of the body's equator.
    pub declination: f64,
    // Radians east of the prime meridian where it is overhead.
    pub overhead_longitude: f64,
    // Radians from its centre to its edge.
    pub angular_radius: f64,
}

impl SkyPosition {
    pub fn at(simulation: &Simulation, body: BodyIndex, other: BodyIndex, time: SimulationTime) -> SkyPosition {
        let mut state = SimulationState::default();
        simulation.state_at(time, &mut state);
        let body_state = state.body(body);
        let offset = state.body(other).position - body_state.position;
        let distance = offset.length();
        let direction = body_state.orientation.inverse() * (offset / distance);

        SkyPosition {
            declination: direction.z.clamp(-1.0, 1.0).asin(),
            overhead_longitude: direction.y.atan2(direction.x),
            angular_radius: (simulation.body(other).radius / distance).min(1.0).asin(),
        }
    }
}
