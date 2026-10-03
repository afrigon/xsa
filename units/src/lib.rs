mod civil_date;
mod simulation_time;
mod time_rate;

pub use simulation_time::SimulationTime;
pub use time_rate::TimeRate;

pub const TICK_RATE_HERTZ: f64 = 60.0;
pub const SECONDS_PER_DAY: f64 = 86_400.0;
pub const SECONDS_PER_JULIAN_CENTURY: f64 = 36_525.0 * SECONDS_PER_DAY;
