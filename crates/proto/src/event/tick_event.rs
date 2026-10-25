use bitcode::{Decode, Encode};
use xsa_units::{SimulationTime, TimeRate};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
/// The simulation advanced by one tick. Sent every tick; between ticks, clients extrapolate time from its rate.
pub struct TickEvent {
    /// The simulation time after the tick.
    pub time: SimulationTime,
    /// The rate time ran at over the tick: the configured one, or a warp's.
    pub rate: TimeRate,
}
