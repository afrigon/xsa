use xsa_units::{SimulationTime, TimeFlow, TimeRate};

pub struct ClockReading {
    pub time: SimulationTime,
    pub time_flow: TimeFlow,
    // The rate time runs at now, a warp's while one runs.
    pub rate: TimeRate,
}
