use super::{
    AVERAGE_MINIMUM_LIT_FRACTION, EXPOSURE_SCALE, HIGHLIGHT_EXPOSED_VALUE, HIGHLIGHT_MINIMUM_LIT_FRACTION,
    HIGHLIGHT_PERCENTILE_FRACTION, HISTOGRAM_BINS, LIT_FLOOR_LUMINANCE, MAXIMUM_EV100, MAXIMUM_LOG2_LUMINANCE,
    METER_SENSITIVITY, MINIMUM_EV100, MINIMUM_LOG2_LUMINANCE, SENSOR_DYNAMIC_RANGE_STOPS, STAR_SURFACE_LUMINANCE,
};

#[derive(Default)]
pub(super) struct Metering {
    average: Option<f32>,
    highlight: Option<f32>,
    star_surface: Option<f32>,
}

impl Metering {
    pub fn from_histogram(histogram: &[u32]) -> Metering {
        let total: u64 = histogram.iter().map(|&count| u64::from(count)).sum();
        let mut metering = Metering::default();

        if total == 0 {
            return metering;
        }

        let first_lit = (1..HISTOGRAM_BINS)
            .find(|&bin| Metering::bin_luminance(bin) >= LIT_FLOOR_LUMINANCE)
            .unwrap_or(HISTOGRAM_BINS);
        let first_star_surface = (first_lit..HISTOGRAM_BINS)
            .find(|&bin| Metering::bin_luminance(bin) >= STAR_SURFACE_LUMINANCE)
            .unwrap_or(HISTOGRAM_BINS);
        let lit_bins = first_lit..first_star_surface;

        metering.star_surface = (first_star_surface..HISTOGRAM_BINS)
            .rev()
            .find(|&bin| histogram[bin] > 0)
            .map(Metering::bin_luminance);

        let lit: u64 = histogram[lit_bins.clone()].iter().map(|&count| u64::from(count)).sum();
        let lit_fraction = lit as f32 / total as f32;

        if lit == 0 || lit_fraction < HIGHLIGHT_MINIMUM_LIT_FRACTION {
            return metering;
        }

        let mut brighter_than = 0;
        let percentile_count = (lit as f32 * HIGHLIGHT_PERCENTILE_FRACTION).ceil() as u64;
        metering.highlight = lit_bins.clone().rev().find_map(|bin| {
            brighter_than += u64::from(histogram[bin]);
            (brighter_than >= percentile_count).then(|| Metering::bin_luminance(bin))
        });

        if lit_fraction >= AVERAGE_MINIMUM_LIT_FRACTION {
            let weighted_log: f64 = lit_bins
                .map(|bin| f64::from(histogram[bin]) * f64::from(Metering::bin_log2_luminance(bin)))
                .sum();
            metering.average = Some((weighted_log / lit as f64).exp2() as f32);
        }

        metering
    }

    // The darkest of the exposures each part of the view calls for: the average lit surface at mid-grey, lit highlights
    // just under white, and a visible star's surface within the sensor's dynamic range. With nothing lit, the dark limit.
    pub fn target_ev100(&self) -> f32 {
        let average = self.average.map(|luminance| (luminance * METER_SENSITIVITY).log2());
        let highlight = self
            .highlight
            .map(|luminance| (luminance / (EXPOSURE_SCALE * HIGHLIGHT_EXPOSED_VALUE)).log2());
        let star_surface = self.star_surface.map(|luminance| {
            (luminance / (EXPOSURE_SCALE * HIGHLIGHT_EXPOSED_VALUE)).log2() - SENSOR_DYNAMIC_RANGE_STOPS
        });
        [average, highlight, star_surface]
            .into_iter()
            .flatten()
            .fold(MINIMUM_EV100, f32::max)
            .clamp(MINIMUM_EV100, MAXIMUM_EV100)
    }

    fn bin_log2_luminance(bin: usize) -> f32 {
        let lit_bins = (HISTOGRAM_BINS - 2) as f32;
        let position = ((bin - 1) as f32 + 0.5) / lit_bins;
        MINIMUM_LOG2_LUMINANCE + position * (MAXIMUM_LOG2_LUMINANCE - MINIMUM_LOG2_LUMINANCE)
    }

    fn bin_luminance(bin: usize) -> f32 {
        Metering::bin_log2_luminance(bin).exp2()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::auto_exposure::{
        EXPOSURE_SCALE, HIGHLIGHT_EXPOSED_VALUE, HISTOGRAM_BINS, MAXIMUM_LOG2_LUMINANCE, MINIMUM_EV100,
        MINIMUM_LOG2_LUMINANCE, SENSOR_DYNAMIC_RANGE_STOPS,
    };

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
        Metering::from_histogram(&histogram(pixels)).target_ev100()
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
}
