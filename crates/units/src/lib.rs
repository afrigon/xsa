mod civil_date;
mod simulation_duration;
mod simulation_time;
mod time_flow;
mod time_rate;
mod time_rate_step;

pub use simulation_duration::SimulationDuration;
pub use simulation_time::SimulationTime;
pub use time_flow::TimeFlow;
pub use time_rate::TimeRate;
pub use time_rate_step::TimeRateStep;

pub const TICK_RATE_HERTZ: f64 = 60.0;
pub const SECONDS_PER_DAY: f64 = 86_400.0;
pub const SECONDS_PER_JULIAN_CENTURY: f64 = 36_525.0 * SECONDS_PER_DAY;
