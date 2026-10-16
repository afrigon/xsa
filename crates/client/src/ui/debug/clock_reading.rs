use xsa_units::{SimulationTime, TimeFlow};

pub struct ClockReading {
    pub time: SimulationTime,
    pub time_flow: TimeFlow,
}
