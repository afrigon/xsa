use std::time::Instant;

// Must match histogram.slang.
pub(super) const HISTOGRAM_BINS: usize = 256;
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
#[derive(Default)]
struct Metering {
    average: Option<f32>,
    highlight: Option<f32>,
    star_surface: Option<f32>,
}

pub(super) struct AutoExposure {
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
        let target = target_ev100(&meter(histogram)) - self.compensation;
        self.ev100 = match elapsed {
            Some(seconds) => adapt(self.ev100, target, seconds),
            None => target,
        };
    }
}

// The darkest of the exposures each part of the view calls for: the average lit surface at mid-grey, lit highlights
// just under white, and a visible star's surface within the sensor's dynamic range. With nothing lit, the dark limit.
fn target_ev100(metering: &Metering) -> f32 {
    let average = metering.average.map(|luminance| (luminance * METER_SENSITIVITY).log2());
    let highlight = metering
        .highlight
        .map(|luminance| (luminance / (EXPOSURE_SCALE * HIGHLIGHT_EXPOSED_VALUE)).log2());
    let star_surface = metering
        .star_surface
        .map(|luminance| (luminance / (EXPOSURE_SCALE * HIGHLIGHT_EXPOSED_VALUE)).log2() - SENSOR_DYNAMIC_RANGE_STOPS);
    [average, highlight, star_surface]
        .into_iter()
        .flatten()
        .fold(MINIMUM_EV100, f32::max)
        .clamp(MINIMUM_EV100, MAXIMUM_EV100)
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

fn bin_luminance(bin: usize) -> f32 {
    bin_log2_luminance(bin).exp2()
}

fn meter(histogram: &[u32]) -> Metering {
    let total: u64 = histogram.iter().map(|&count| u64::from(count)).sum();
    let mut metering = Metering::default();
    if total == 0 {
        return metering;
    }
    let first_lit = (1..HISTOGRAM_BINS)
        .find(|&bin| bin_luminance(bin) >= LIT_FLOOR_LUMINANCE)
        .unwrap_or(HISTOGRAM_BINS);
    let first_star_surface = (first_lit..HISTOGRAM_BINS)
        .find(|&bin| bin_luminance(bin) >= STAR_SURFACE_LUMINANCE)
        .unwrap_or(HISTOGRAM_BINS);
    let lit_bins = first_lit..first_star_surface;

    metering.star_surface = (first_star_surface..HISTOGRAM_BINS)
        .rev()
        .find(|&bin| histogram[bin] > 0)
        .map(bin_luminance);

    let lit: u64 = histogram[lit_bins.clone()].iter().map(|&count| u64::from(count)).sum();
    let lit_fraction = lit as f32 / total as f32;
    if lit == 0 || lit_fraction < HIGHLIGHT_MINIMUM_LIT_FRACTION {
        return metering;
    }

    let mut brighter_than = 0;
    let percentile_count = (lit as f32 * HIGHLIGHT_PERCENTILE_FRACTION).ceil() as u64;
    metering.highlight = lit_bins.clone().rev().find_map(|bin| {
        brighter_than += u64::from(histogram[bin]);
        (brighter_than >= percentile_count).then(|| bin_luminance(bin))
    });

    if lit_fraction >= AVERAGE_MINIMUM_LIT_FRACTION {
        let weighted_log: f64 = lit_bins
            .map(|bin| f64::from(histogram[bin]) * f64::from(bin_log2_luminance(bin)))
            .sum();
        metering.average = Some((weighted_log / lit as f64).exp2() as f32);
    }
    metering
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

    fn ev100(pixels: &[Pixels]) -> f32 {
        target_ev100(&meter(&histogram(pixels)))
    }

    fn exposed(luminance: f32, ev100: f32) -> f32 {
        luminance / (EXPOSURE_SCALE * ev100.exp2())
    }

    const SKY: f32 = 1e-4;
    const SUNLIT: f32 = 5_000.0;
    const SUN_DISK: f32 = 1.9e9;

    #[test]
    fn a_large_sunlit_planet_sits_near_mid_grey() {
        let ev100 = ev100(&[
            Pixels {
                luminance: SKY,
                count: 600_000,
            },
            Pixels {
                luminance: SUNLIT,
                count: 400_000,
            },
        ]);
        assert!((0.05..0.5).contains(&exposed(SUNLIT, ev100)), "EV100 {ev100}");
    }

    #[test]
    fn a_thin_crescent_is_kept_under_white() {
        let ev100 = ev100(&[
            Pixels {
                luminance: SKY,
                count: 995_000,
            },
            Pixels {
                luminance: SUNLIT,
                count: 5_000,
            },
        ]);
        let value = exposed(SUNLIT, ev100);
        assert!(
            (0.5..=HIGHLIGHT_EXPOSED_VALUE * 1.2).contains(&value),
            "crescent at {value}, EV100 {ev100}"
        );
    }

    #[test]
    fn a_distant_small_planet_is_kept_under_white() {
        let ev100 = ev100(&[
            Pixels {
                luminance: SKY,
                count: 999_800,
            },
            Pixels {
                luminance: SUNLIT,
                count: 200,
            },
        ]);
        assert!(exposed(SUNLIT, ev100) <= HIGHLIGHT_EXPOSED_VALUE * 1.2, "EV100 {ev100}");
    }

    #[test]
    fn the_sun_in_view_stays_within_the_sensor_range() {
        let ev100 = ev100(&[
            Pixels {
                luminance: SKY,
                count: 994_900,
            },
            Pixels {
                luminance: SUNLIT,
                count: 5_000,
            },
            Pixels {
                luminance: SUN_DISK,
                count: 100,
            },
        ]);
        let stops_over_white = (exposed(SUN_DISK, ev100) / HIGHLIGHT_EXPOSED_VALUE).log2();
        assert!(
            stops_over_white <= SENSOR_DYNAMIC_RANGE_STOPS + 0.2,
            "Sun {stops_over_white} stops over white"
        );
    }

    #[test]
    fn open_space_stops_at_the_dark_limit() {
        assert_eq!(
            ev100(&[Pixels {
                luminance: SKY,
                count: 1_000_000
            }]),
            MINIMUM_EV100
        );
        assert_eq!(
            ev100(&[Pixels {
                luminance: 0.0,
                count: 1_000_000
            }]),
            MINIMUM_EV100
        );
    }

    #[test]
    fn bright_patches_on_a_dark_body_stay_under_white() {
        let ev100 = ev100(&[
            Pixels {
                luminance: 50.0,
                count: 80_000,
            },
            Pixels {
                luminance: SUNLIT,
                count: 20_000,
            },
        ]);
        assert!(exposed(SUNLIT, ev100) <= HIGHLIGHT_EXPOSED_VALUE * 1.2, "EV100 {ev100}");
    }

    #[test]
    fn adaptation_closes_half_the_gap_in_one_half_life() {
        let brighter = adapt(0.0, 10.0, BRIGHTENING_HALF_LIFE_SECONDS);
        let darker = adapt(10.0, 0.0, DARKENING_HALF_LIFE_SECONDS);
        assert!((brighter - 5.0).abs() < 1e-4 && (darker - 5.0).abs() < 1e-4);
    }
}
