use bitcode::{Decode, Encode};
use xsa_units::{SimulationTime, TimeFlow, TimeRate};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// Simulation time jumped, or the rate it runs at changed.
pub struct TimeChangedEvent {
    /// The simulation time after the change.
    pub time: SimulationTime,
    /// How time runs from now on.
    pub time_flow: TimeFlow,
    /// The rate time runs at from now on, until the next tick says otherwise.
    pub rate: TimeRate,
}
