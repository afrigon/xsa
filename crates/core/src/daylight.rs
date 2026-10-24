mod daylight_error;
mod star_sky;
mod time_of_day;

pub use daylight_error::DaylightError;
pub use star_sky::StarSky;
pub use time_of_day::TimeOfDay;

use std::f64::consts::{PI, TAU};

use xsa_units::{SimulationDuration, SimulationTime, TICK_RATE_HERTZ};

use crate::simulation::{BodyIndex, Simulation};

// How far apart the two skies the hour angle's rate is measured between are.
const RATE_SAMPLE_SECONDS: f64 = 60.0;
const MAXIMUM_REFINEMENTS: usize = 16;
// A time is found once a refinement moves it by less than a tick.
const TIME_TOLERANCE_SECONDS: f64 = 1.0 / TICK_RATE_HERTZ;

// When and where a time of day falls on a body, against one of its stars. The star's hour angle at a longitude is how
// far the body has turned that longitude past the star: 0 at noon, π at midnight. Rising and setting come from the
// sunrise equation, refined as the star's declination drifts along the orbit.
pub struct Daylight<'a> {
    simulation: &'a Simulation,
    body: BodyIndex,
    star: BodyIndex,
}

impl<'a> Daylight<'a> {
    pub fn new(simulation: &'a Simulation, body: BodyIndex, star: BodyIndex) -> Daylight<'a> {
        Daylight { simulation, body, star }
    }

    pub fn sky(&self, time: SimulationTime) -> StarSky {
        StarSky::at(self.simulation, self.body, self.star, time)
    }

    // Radians east of the prime meridian where `time_of_day` falls at `latitude` at `time`.
    pub fn longitude(&self, time_of_day: TimeOfDay, latitude: f64, time: SimulationTime) -> Result<f64, DaylightError> {
        let rate = self.hour_angle_rate(time)?;
        let sky = self.sky(time);
        let hour_angle = self.hour_angle_of(time_of_day, latitude, &sky, rate)?;

        Ok(Daylight::wrapped(sky.substellar_longitude + hour_angle))
    }

    // The first time after `after` that `time_of_day` falls at `latitude` and `longitude`.
    pub fn next(
        &self,
        time_of_day: TimeOfDay,
        latitude: f64,
        longitude: f64,
        after: SimulationTime,
    ) -> Result<SimulationTime, DaylightError> {
        let rate = self.hour_angle_rate(after)?;
        let sky = self.sky(after);
        let hour_angle = longitude - sky.substellar_longitude;
        let target = self.hour_angle_of(time_of_day, latitude, &sky, rate)?;
        let wait = ((target - hour_angle) * rate.signum()).rem_euclid(TAU) / rate.abs();
        let time = self.refine(time_of_day, latitude, longitude, after.advanced_by(seconds(wait)), rate)?;

        if time.seconds_since(after) > TIME_TOLERANCE_SECONDS {
            return Ok(time);
        }

        // The one found is `after` itself, or refining crossed back over it: the next one comes a turn later.
        self.refine(
            time_of_day,
            latitude,
            longitude,
            time.advanced_by(seconds(TAU / rate.abs())),
            rate,
        )
    }

    // Moves `time` onto `time_of_day`, the declination and the hour angle's target taken at each guess.
    fn refine(
        &self,
        time_of_day: TimeOfDay,
        latitude: f64,
        longitude: f64,
        mut time: SimulationTime,
        rate: f64,
    ) -> Result<SimulationTime, DaylightError> {
        for _ in 0..MAXIMUM_REFINEMENTS {
            let sky = self.sky(time);
            let hour_angle = longitude - sky.substellar_longitude;
            let target = self.hour_angle_of(time_of_day, latitude, &sky, rate)?;
            let correction = Daylight::wrapped(target - hour_angle) / rate;
            time = time.advanced_by(seconds(correction));

            if correction.abs() < TIME_TOLERANCE_SECONDS {
                return Ok(time);
            }
        }

        Err(DaylightError::DidNotConverge { time_of_day })
    }

    // Radians per second the star's hour angle advances: negative on a body spinning against its orbit (Venus).
    fn hour_angle_rate(&self, time: SimulationTime) -> Result<f64, DaylightError> {
        let before = self.sky(time).substellar_longitude;
        let after = self
            .sky(time.advanced_by(seconds(RATE_SAMPLE_SECONDS)))
            .substellar_longitude;
        let rate = -Daylight::wrapped(after - before) / RATE_SAMPLE_SECONDS;

        if rate == 0.0 || !rate.is_finite() {
            return Err(DaylightError::StarFixed);
        }

        Ok(rate)
    }

    // The hour angle `time_of_day` falls at, given the direction the hour angle advances in.
    fn hour_angle_of(
        &self,
        time_of_day: TimeOfDay,
        latitude: f64,
        sky: &StarSky,
        rate: f64,
    ) -> Result<f64, DaylightError> {
        let (elevation, rising) = match time_of_day {
            TimeOfDay::Noon => return Ok(0.0),
            TimeOfDay::Midnight => return Ok(PI),
            TimeOfDay::Sunrise => (-sky.angular_radius, true),
            TimeOfDay::Sunset => (-sky.angular_radius, false),
            TimeOfDay::Dawn => (-self.twilight_depression(sky), true),
            TimeOfDay::Dusk => (-self.twilight_depression(sky), false),
        };
        let cosine =
            (elevation.sin() - latitude.sin() * sky.declination.sin()) / (latitude.cos() * sky.declination.cos());

        if !(-1.0..=1.0).contains(&cosine) {
            return Err(DaylightError::NoCrossing { time_of_day });
        }

        // The star climbs while the hour angle moves towards 0: before noon when it advances, after when it recedes.
        let side = if rising { -rate.signum() } else { rate.signum() };

        Ok(side * cosine.acos())
    }

    // Below the horizon where twilight ends; without an atmosphere's twilight, where the star's edge sets.
    fn twilight_depression(&self, sky: &StarSky) -> f64 {
        self.simulation
            .body(self.body)
            .atmosphere
            .and_then(|atmosphere| atmosphere.twilight_depression)
            .unwrap_or(sky.angular_radius)
    }

    fn wrapped(angle: f64) -> f64 {
        (angle + PI).rem_euclid(TAU) - PI
    }
}

fn seconds(seconds: f64) -> SimulationDuration {
    SimulationDuration { seconds }
}
