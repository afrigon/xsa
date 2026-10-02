use std::time::Instant;

// Must match histogram.slang.
pub(super) const HISTOGRAM_BINS: usize = 256;
const MINIMUM_LOG2_LUMINANCE: f32 = -16.0;
const MAXIMUM_LOG2_LUMINANCE: f32 = 32.0;

// Above the brightest stars of the sky (about 0.05 cd/m²), so starfields never count as lit content.
const LIT_FLOOR_LUMINANCE: f32 = 0.1;
const MINIMUM_LIT_FRACTION: f32 = 0.01;
const BRIGHTEST_EXCLUDED_FRACTION: f32 = 0.02;
const MINIMUM_EV100: f32 = -6.0;
const MAXIMUM_EV100: f32 = 20.0;
const BRIGHTENING_HALF_LIFE_SECONDS: f32 = 0.1;
const DARKENING_HALF_LIFE_SECONDS: f32 = 1.5;
// ISO 100 and the reflected-light meter calibration constant K = 12.5.
const METER_SENSITIVITY: f32 = 100.0 / 12.5;
// Exposed value of the brightest metered pixels: just under AgX's white point (about 2.9).
const HIGHLIGHT_EXPOSED_VALUE: f32 = 2.0;
// The exposure for an EV100 is 1 / (1.2 × 2^EV100).
const EXPOSURE_SCALE: f32 = 1.2;

struct Metering {
    average: f32,
    highlight: f32,
}

pub(super) struct AutoExposure {
    enabled: bool,
    instant: bool,
    ev100: f32,
    manual_ev100: f32,
    compensation: f32,
    last_update: Option<Instant>,
}

impl AutoExposure {
    pub fn new(initial_ev100: f32) -> Self {
        Self {
            enabled: true,
            instant: false,
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

    pub fn toggle_instant(&mut self) -> bool {
        self.instant = !self.instant;
        self.instant
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
        let Some(metering) = meter(histogram) else {
            return;
        };
        let target = target_ev100(&metering) - self.compensation;
        self.ev100 = match elapsed {
            Some(seconds) if !self.instant => adapt(self.ev100, target, seconds),
            _ => target,
        };
    }
}

// The darker of two exposures: the average lit surface at mid-grey, and the highlights just under white, so
// dark bodies with bright patches (Earth's oceans and ice) do not clip.
fn target_ev100(metering: &Metering) -> f32 {
    let average = (metering.average * METER_SENSITIVITY).log2();
    let highlight = (metering.highlight / (EXPOSURE_SCALE * HIGHLIGHT_EXPOSED_VALUE)).log2();
    average.max(highlight).clamp(MINIMUM_EV100, MAXIMUM_EV100)
}

fn adapt(current: f32, target: f32, seconds: f32) -> f32 {
    let half_life = if target > current {
        BRIGHTENING_HALF_LIFE_SECONDS
    } else {
        DARKENING_HALF_LIFE_SECONDS
    };
    current + (target - current) * (1.0 - 0.5_f32.powf(seconds / half_life))
}

fn bin_log2_luminance(bin: usize) -> f32 {
    let lit_bins = (HISTOGRAM_BINS - 2) as f32;
    let position = ((bin - 1) as f32 + 0.5) / lit_bins;
    MINIMUM_LOG2_LUMINANCE + position * (MAXIMUM_LOG2_LUMINANCE - MINIMUM_LOG2_LUMINANCE)
}

// Average luminance of the lit part of the view, ignoring black space and the brightest pixels (the Sun's disk);
// when almost nothing is lit, the whole view counts.
fn meter(histogram: &[u32]) -> Option<Metering> {
    let total: u64 = histogram.iter().map(|&count| u64::from(count)).sum();
    let floor = LIT_FLOOR_LUMINANCE.log2();
    let first_lit = (1..HISTOGRAM_BINS)
        .find(|&bin| bin_log2_luminance(bin) >= floor)
        .unwrap_or(HISTOGRAM_BINS);
    let lit: u64 = histogram[first_lit..].iter().map(|&count| u64::from(count)).sum();
    let first_metered = if lit as f32 >= MINIMUM_LIT_FRACTION * total as f32 {
        first_lit
    } else {
        1
    };

    let metered: u64 = histogram[first_metered..].iter().map(|&count| u64::from(count)).sum();
    let mut excluded = (metered as f32 * BRIGHTEST_EXCLUDED_FRACTION) as u64;
    let mut weighted_log = 0.0;
    let mut counted = 0;
    let mut highlight = None;
    for bin in (first_metered..HISTOGRAM_BINS).rev() {
        let count = u64::from(histogram[bin]);
        let skipped = count.min(excluded);
        excluded -= skipped;
        let kept = count - skipped;
        if kept > 0 && highlight.is_none() {
            highlight = Some(bin_log2_luminance(bin).exp2());
        }
        weighted_log += kept as f64 * f64::from(bin_log2_luminance(bin));
        counted += kept;
    }
    let highlight = highlight?;
    Some(Metering {
        average: (weighted_log / counted as f64).exp2() as f32,
        highlight,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bin_of(luminance: f32) -> usize {
        let position = (luminance.log2() - MINIMUM_LOG2_LUMINANCE) / (MAXIMUM_LOG2_LUMINANCE - MINIMUM_LOG2_LUMINANCE);
        1 + (position.clamp(0.0, 1.0) * (HISTOGRAM_BINS - 2) as f32) as usize
    }

    fn histogram(pixels: &[Pixels]) -> Vec<u32> {
        let mut bins = vec![0; HISTOGRAM_BINS];
        for group in pixels {
            let bin = if group.luminance > 0.0 {
                bin_of(group.luminance)
            } else {
                0
            };
            bins[bin] += group.count;
        }
        bins
    }

    struct Pixels {
        luminance: f32,
        count: u32,
    }

    #[test]
    fn small_sunlit_planet_in_black_space_is_metered_on_the_planet() {
        let view = histogram(&[
            Pixels {
                luminance: 1e-4,
                count: 950_000,
            },
            Pixels {
                luminance: 10_000.0,
                count: 50_000,
            },
        ]);
        let luminance = meter(&view).unwrap().average;
        assert!((5_000.0..20_000.0).contains(&luminance), "{luminance} cd/m²");
    }

    #[test]
    fn open_space_stops_at_the_dark_limit() {
        let view = histogram(&[Pixels {
            luminance: 1e-4,
            count: 1_000_000,
        }]);
        assert_eq!(target_ev100(&meter(&view).unwrap()), MINIMUM_EV100);
    }

    #[test]
    fn the_brightest_pixels_do_not_set_the_exposure() {
        let view = histogram(&[
            Pixels {
                luminance: 10_000.0,
                count: 990_000,
            },
            Pixels {
                luminance: 2e9,
                count: 10_000,
            },
        ]);
        let luminance = meter(&view).unwrap().average;
        assert!(luminance < 20_000.0, "{luminance} cd/m²");
    }

    #[test]
    fn a_black_view_keeps_the_current_exposure() {
        let view = histogram(&[Pixels {
            luminance: 0.0,
            count: 1_000_000,
        }]);
        assert!(meter(&view).is_none());
    }

    #[test]
    fn bright_patches_on_a_dark_body_stay_under_white() {
        let view = histogram(&[
            Pixels {
                luminance: 50.0,
                count: 80_000,
            },
            Pixels {
                luminance: 5_000.0,
                count: 20_000,
            },
        ]);
        let ev100 = target_ev100(&meter(&view).unwrap());
        let exposed = 5_000.0 / (EXPOSURE_SCALE * ev100.exp2());
        assert!(
            exposed <= HIGHLIGHT_EXPOSED_VALUE * 1.2,
            "land exposed to {exposed} at EV100 {ev100}"
        );
    }

    #[test]
    fn adaptation_closes_half_the_gap_in_one_half_life() {
        let brighter = adapt(0.0, 10.0, BRIGHTENING_HALF_LIFE_SECONDS);
        let darker = adapt(10.0, 0.0, DARKENING_HALF_LIFE_SECONDS);
        assert!((brighter - 5.0).abs() < 1e-4 && (darker - 5.0).abs() < 1e-4);
    }
}
