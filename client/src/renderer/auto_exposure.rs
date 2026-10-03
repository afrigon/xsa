mod metering;

use std::time::Instant;

use metering::Metering;

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
const BRIGHTENING_HALF_LIFE_SECONDS: f32 = 0.1;
const DARKENING_HALF_LIFE_SECONDS: f32 = 1.5;
// ISO 100 and the reflected-light meter calibration constant K = 12.5.
const METER_SENSITIVITY: f32 = 100.0 / 12.5;
// Exposed value of lit highlights: just under AgX's white point (about 2.9).
const HIGHLIGHT_EXPOSED_VALUE: f32 = 2.0;
// A camera sensor records about 14 stops above its white point; a visible star's surface is kept within that range.
const SENSOR_DYNAMIC_RANGE_STOPS: f32 = 14.0;
// The exposure for an EV100 is 1 / (1.2 × 2^EV100).
const EXPOSURE_SCALE: f32 = 1.2;

pub struct AutoExposure {
    enabled: bool,
    ev100: f32,
    manual_ev100: f32,
    compensation: f32,
    last_update: Option<Instant>,
}

impl AutoExposure {
    pub fn new(initial_ev100: f32) -> Self {
        Self {
            enabled: true,
            ev100: initial_ev100,
            manual_ev100: initial_ev100,
            compensation: 0.0,
            last_update: None,
        }
    }

    pub fn ev100(&self) -> f32 {
        if self.enabled { self.ev100 } else { self.manual_ev100 }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn compensation(&self) -> f32 {
        self.compensation
    }

    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        if self.enabled {
            self.ev100 = self.manual_ev100;
        } else {
            self.manual_ev100 = self.ev100;
        }
        self.enabled
    }

    // Brightens the image by `stops`: shifts the compensation in auto mode, the exposure itself in manual mode.
    pub fn adjust(&mut self, stops: f32) {
        if self.enabled {
            self.compensation += stops;
        } else {
            self.manual_ev100 -= stops;
        }
    }

    pub fn update(&mut self, histogram: &[u32]) {
        let now = Instant::now();
        let elapsed = self.last_update.map(|last| now.duration_since(last).as_secs_f32());
        self.last_update = Some(now);
        if !self.enabled {
            return;
        }
        let target = Metering::from_histogram(histogram).target_ev100() - self.compensation;
        self.ev100 = match elapsed {
            Some(seconds) => AutoExposure::adapt(self.ev100, target, seconds),
            None => target,
        };
    }

    // Scales luminance (cd/m²) so that a scene metered at this EV100 lands mid-range, as a camera with ISO 100 would.
    pub fn exposure(&self) -> f32 {
        1.0 / (EXPOSURE_SCALE * self.ev100().exp2())
    }

    fn adapt(current: f32, target: f32, seconds: f32) -> f32 {
        let half_life = if target > current {
            BRIGHTENING_HALF_LIFE_SECONDS
        } else {
            DARKENING_HALF_LIFE_SECONDS
        };
        current + (target - current) * (1.0 - 0.5_f32.powf(seconds / half_life))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptation_closes_half_the_gap_in_one_half_life() {
        let brighter = AutoExposure::adapt(0.0, 10.0, BRIGHTENING_HALF_LIFE_SECONDS);
        let darker = AutoExposure::adapt(10.0, 0.0, DARKENING_HALF_LIFE_SECONDS);
        assert!((brighter - 5.0).abs() < 1e-4 && (darker - 5.0).abs() < 1e-4);
    }
}
