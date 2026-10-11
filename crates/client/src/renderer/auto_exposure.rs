use std::time::Instant;

use crate::config::{ExposureConfig, ExposureMode};

// Must match histogram.slang.
pub(in crate::renderer) const HISTOGRAM_BINS: usize = 256;
const ISO_SENSITIVITY: f32 = 100.0;
// The incident-light meter calibration constant C, in lux·s.
const INCIDENT_METER_CALIBRATION: f32 = 250.0;
// The exposure for an EV100 is 1 / (1.2 × 2^EV100).
const EXPOSURE_SCALE: f32 = 1.2;
// Eyes adapt to brighter scenes far faster than to darker ones.
const DARK_TO_LIGHT_HALF_LIFE_SECONDS: f32 = 0.5;
const LIGHT_TO_DARK_HALF_LIFE_SECONDS: f32 = 2.5;

pub struct AutoExposure {
    config: ExposureConfig,
    anchor_ev100: f32,
    adapted_ev100: f32,
    last_update: Option<Instant>,
    duration_scale: f32,
}

impl AutoExposure {
    pub fn new(config: ExposureConfig, initial_ev100: f32) -> Self {
        Self {
            config,
            anchor_ev100: initial_ev100,
            adapted_ev100: initial_ev100,
            last_update: None,
            duration_scale: 1.0,
        }
    }

    pub fn configure(&mut self, config: &ExposureConfig, duration_scale: f32) {
        self.config = config.clone();
        self.duration_scale = duration_scale;
    }

    pub fn ev100(&self) -> f32 {
        match self.config.mode {
            ExposureMode::EyeAdaptation => self.adapted_ev100,
            ExposureMode::Manual => self.config.ev100,
        }
    }

    // The exposure an incident-light meter at the camera calls for.
    pub fn anchor_ev100(&self) -> f32 {
        self.anchor_ev100
    }

    // Illuminance at the camera, in lux.
    pub fn update(&mut self, illuminance: f32) {
        let now = Instant::now();
        let elapsed = self.last_update.map(|last| now.duration_since(last).as_secs_f32());
        self.last_update = Some(now);
        self.anchor_ev100 = (illuminance * ISO_SENSITIVITY / INCIDENT_METER_CALIBRATION).log2();
        self.adapted_ev100 = match elapsed {
            Some(seconds) => self.adapt(self.adapted_ev100, self.anchor_ev100, seconds),
            None => self.anchor_ev100,
        };
    }

    // Scales luminance (cd/m²) so that a scene metered at this EV100 lands mid-range, as a camera with ISO 100 would.
    pub fn exposure(&self) -> f32 {
        1.0 / (EXPOSURE_SCALE * self.ev100().exp2())
    }

    // A higher target EV100 means the scene got brighter.
    fn adapt(&self, current: f32, target: f32, seconds: f32) -> f32 {
        let half_life = if target > current {
            DARK_TO_LIGHT_HALF_LIFE_SECONDS
        } else {
            LIGHT_TO_DARK_HALF_LIFE_SECONDS
        } * self.duration_scale;

        if half_life <= 0.0 {
            return target;
        }

        current + (target - current) * (1.0 - 0.5_f32.powf(seconds / half_life))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    const SUNLIGHT_AT_ONE_ASTRONOMICAL_UNIT: f32 = 128_000.0;

    #[test]
    fn sunlight_at_one_astronomical_unit_meters_like_a_sunny_day() {
        let mut exposure = AutoExposure::new(Config::default().render.exposure, 0.0);
        exposure.update(SUNLIGHT_AT_ONE_ASTRONOMICAL_UNIT);
        assert!(
            (exposure.anchor_ev100() - 15.6).abs() < 0.1,
            "EV100 {}",
            exposure.anchor_ev100()
        );
    }

    #[test]
    fn adaptation_closes_half_the_gap_in_one_half_life() {
        let exposure = AutoExposure::new(Config::default().render.exposure, 0.0);
        let brighter = exposure.adapt(0.0, 10.0, DARK_TO_LIGHT_HALF_LIFE_SECONDS);
        let darker = exposure.adapt(10.0, 0.0, LIGHT_TO_DARK_HALF_LIFE_SECONDS);
        assert!((brighter - 5.0).abs() < 1e-4 && (darker - 5.0).abs() < 1e-4);
    }

    #[test]
    fn a_zero_duration_scale_adapts_at_once() {
        let config = Config::default().render.exposure;
        let mut exposure = AutoExposure::new(config.clone(), 0.0);
        exposure.configure(&config, 0.0);
        assert_eq!(exposure.adapt(0.0, 10.0, 0.0), 10.0);
        assert_eq!(exposure.adapt(10.0, 0.0, 0.0), 0.0);
    }
}
