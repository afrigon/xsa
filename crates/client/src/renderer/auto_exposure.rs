mod metering;

use std::time::Instant;

use metering::Metering;

use super::FRAMES_IN_FLIGHT;

use crate::config::{ExposureConfig, ExposureMode};

// Must match histogram.slang.
pub(in crate::renderer) const HISTOGRAM_BINS: usize = 256;
const MINIMUM_LOG2_LUMINANCE: f32 = -16.0;
const MAXIMUM_LOG2_LUMINANCE: f32 = 32.0;

// Pixels are classified by what they physically are. Below LIT_FLOOR: starfield and night sides (the brightest
// stars of the sky are about 0.05 cd/m²). Above STAR_SURFACE: emitted light; the brightest reflected sunlight, a
// white surface at Mercury, stays below 3·10⁵ cd/m².
const LIT_FLOOR_LUMINANCE: f32 = 0.1;
const STAR_SURFACE_LUMINANCE: f32 = 1e7;
const AVERAGE_MINIMUM_LIT_FRACTION: f32 = 0.01;
const HIGHLIGHT_MINIMUM_LIT_FRACTION: f32 = 1e-4;
// The highlight is the 99th percentile of lit pixels, so a few hot texels do not set the exposure.
const HIGHLIGHT_PERCENTILE_FRACTION: f32 = 0.01;
const MINIMUM_EV100: f32 = -6.0;
const MAXIMUM_EV100: f32 = 20.0;
// ISO 100 and the reflected-light meter calibration constant K = 12.5.
const METER_SENSITIVITY: f32 = 100.0 / 12.5;
// Exposed value of lit highlights: just under AgX's white point (about 2.9).
const HIGHLIGHT_EXPOSED_VALUE: f32 = 2.0;
// A camera sensor records about 14 stops above its white point; a visible star's surface is kept within that range.
const SENSOR_DYNAMIC_RANGE_STOPS: f32 = 14.0;
// The exposure for an EV100 is 1 / (1.2 × 2^EV100).
const EXPOSURE_SCALE: f32 = 1.2;

pub struct AutoExposure {
    config: ExposureConfig,
    metered_ev100: f32,
    last_update: Option<Instant>,
    duration_scale: f32,
}

impl AutoExposure {
    pub fn new(config: ExposureConfig, initial_ev100: f32) -> Self {
        Self {
            config,
            metered_ev100: initial_ev100,
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
            ExposureMode::EyeAdaptation => self.metered_ev100,
            ExposureMode::Manual => self.config.ev100,
        }
    }

    // A frame's histogram is read back when its frame slot comes round again.
    pub fn settling_frames(&self) -> u32 {
        match self.config.mode {
            ExposureMode::EyeAdaptation => FRAMES_IN_FLIGHT as u32,
            ExposureMode::Manual => 0,
        }
    }

    pub fn update(&mut self, histogram: &[u32]) {
        let now = Instant::now();
        let elapsed = self.last_update.map(|last| now.duration_since(last).as_secs_f32());
        self.last_update = Some(now);

        if self.config.mode == ExposureMode::Manual {
            return;
        }

        let target = Metering::from_histogram(histogram).target_ev100() - self.config.compensation;
        self.metered_ev100 = match elapsed {
            Some(seconds) => self.adapt(self.metered_ev100, target, seconds),
            None => target,
        };
    }

    // Scales luminance (cd/m²) so that a scene metered at this EV100 lands mid-range, as a camera with ISO 100 would.
    pub fn exposure(&self) -> f32 {
        1.0 / (EXPOSURE_SCALE * self.ev100().exp2())
    }

    // A higher target EV100 means the scene got brighter.
    fn adapt(&self, current: f32, target: f32, seconds: f32) -> f32 {
        let adaptation = &self.config.adaptation;
        let half_life = if target > current {
            adaptation.dark_to_light_half_life_seconds
        } else {
            adaptation.light_to_dark_half_life_seconds
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

    #[test]
    fn adaptation_closes_half_the_gap_in_one_half_life() {
        let config = Config::default().render.exposure;
        let adaptation = config.adaptation.clone();
        let exposure = AutoExposure::new(config, 0.0);
        let brighter = exposure.adapt(0.0, 10.0, adaptation.dark_to_light_half_life_seconds);
        let darker = exposure.adapt(10.0, 0.0, adaptation.light_to_dark_half_life_seconds);
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
