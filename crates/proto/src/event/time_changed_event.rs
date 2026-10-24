use bitcode::{Decode, Encode};
use xsa_units::{SimulationTime, TimeFlow};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Simulation time jumped, or the rate it runs at changed.
pub struct TimeChangedEvent {
    /// The simulation time after the change.
    pub time: SimulationTime,
    /// How time runs from now on.
    pub time_flow: TimeFlow,
}
