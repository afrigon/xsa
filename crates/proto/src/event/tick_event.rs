use bitcode::{Decode, Encode};
use xsa_units::SimulationTime;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// The simulation advanced by one tick. Sent every tick; between ticks, clients extrapolate time from the rate.
pub struct TickEvent {
    /// The simulation time after the tick.
    pub time: SimulationTime,
}
