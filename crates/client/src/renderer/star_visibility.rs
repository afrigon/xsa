use std::time::Instant;

use super::AutoExposure;

// Must match histogram.slang.
pub(in crate::renderer) const HISTOGRAM_BINS: usize = 256;
const MINIMUM_LOG2_LUMINANCE: f32 = -16.0;
const MAXIMUM_LOG2_LUMINANCE: f32 = 32.0;
// Average exposed brightness of the frame: Earth filling most of it is about 0.01, the full Moon filling it about 0.1,
// the Sun's disk about 0.7.
const STARS_FULLY_VISIBLE_BRIGHTNESS: f32 = 0.01;
const STARS_HIDDEN_BRIGHTNESS: f32 = 0.3;
// Eyes take seconds to adapt to the dark, and a fraction of one to bright light.
const STARS_FADE_IN_HALF_LIFE_SECONDS: f32 = 2.0;
const STARS_FADE_OUT_HALF_LIFE_SECONDS: f32 = 0.1;
// The starfield's own exposure, chosen for looks rather than physics: it does not follow the scene's.
const STARFIELD_EV100: f32 = -7.0;

pub struct StarVisibility {
    visibility: Option<f32>,
    light: f32,
    last_update: Option<Instant>,
    duration_scale: f32,
}

impl StarVisibility {
    pub fn new() -> StarVisibility {
        StarVisibility {
            visibility: None,
            light: 0.0,
            last_update: None,
            duration_scale: 1.0,
        }
    }

    pub fn configure(&mut self, duration_scale: f32) {
        self.duration_scale = duration_scale;
    }

    // The frame's exposed luminance, binned by histogram.slang.
    pub fn update(&mut self, histogram: &[u32]) {
        let now = Instant::now();
        let elapsed = self.last_update.map(|last| now.duration_since(last).as_secs_f32());
        self.last_update = Some(now);
        self.light = StarVisibility::average_brightness(histogram);
        let target = StarVisibility::target_visibility(self.light);
        self.visibility = Some(match (self.visibility, elapsed) {
            (Some(current), Some(seconds)) => self.adapt(current, target, seconds),
            _ => target,
        });
    }

    // Stars stay hidden until the first frame is measured.
    pub fn visibility(&self) -> f32 {
        self.visibility.unwrap_or(0.0)
    }

    // The average exposed brightness of the last measured frame.
    pub fn light(&self) -> f32 {
        self.light
    }

    pub fn starfield_exposure(&self) -> f32 {
        self.visibility() * AutoExposure::exposure_for(STARFIELD_EV100)
    }

    fn target_visibility(brightness: f32) -> f32 {
        let visible = STARS_FULLY_VISIBLE_BRIGHTNESS.log2();
        let hidden = STARS_HIDDEN_BRIGHTNESS.log2();
        let progress = ((brightness.log2() - visible) / (hidden - visible)).clamp(0.0, 1.0);

        1.0 - progress * progress * (3.0 - 2.0 * progress)
    }

    // Bright light hides the stars quickly; darkness reveals them slowly.
    fn adapt(&self, current: f32, target: f32, seconds: f32) -> f32 {
        let half_life = if target < current {
            STARS_FADE_OUT_HALF_LIFE_SECONDS
        } else {
            STARS_FADE_IN_HALF_LIFE_SECONDS
        } * self.duration_scale;

        if half_life <= 0.0 {
            return target;
        }

        current + (target - current) * (1.0 - 0.5_f32.powf(seconds / half_life))
    }

    // Anything darker than the first bin is unmeasurable, so the histogram's floor is the darkest average there is.
    fn average_brightness(histogram: &[u32]) -> f32 {
        let total: u64 = histogram.iter().map(|&count| u64::from(count)).sum();

        if total == 0 {
            return MINIMUM_LOG2_LUMINANCE.exp2();
        }

        let sum: f64 = histogram
            .iter()
            .enumerate()
            .skip(1)
            .map(|(bin, &count)| f64::from(count) * f64::from(StarVisibility::bin_luminance(bin)))
            .sum();

        ((sum / total as f64) as f32).max(MINIMUM_LOG2_LUMINANCE.exp2())
    }

    // Bin 0 holds unlit pixels; bins 1 to 255 split the log2 luminance range evenly.
    fn bin_luminance(bin: usize) -> f32 {
        let lit_bins = (HISTOGRAM_BINS - 2) as f32;
        let position = ((bin - 1) as f32 + 0.5) / lit_bins;
        (MINIMUM_LOG2_LUMINANCE + position * (MAXIMUM_LOG2_LUMINANCE - MINIMUM_LOG2_LUMINANCE)).exp2()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIXELS: u32 = 2_000_000;
    const SUNLIT_EARTH: f32 = 0.01;
    const FULL_MOON: f32 = 0.2;
    const SUN_DISK: f32 = 30_000.0;
    const SUN_DISK_PIXELS: u32 = 47;

    struct Pixels {
        brightness: f32,
        count: u32,
    }

    fn bin_of(brightness: f32) -> usize {
        let position = (brightness.log2() - MINIMUM_LOG2_LUMINANCE) / (MAXIMUM_LOG2_LUMINANCE - MINIMUM_LOG2_LUMINANCE);
        1 + (position.clamp(0.0, 1.0) * (HISTOGRAM_BINS - 2) as f32) as usize
    }

    fn visibility(pixels: &[Pixels]) -> f32 {
        let mut histogram = vec![0; HISTOGRAM_BINS];
        let lit: u32 = pixels.iter().map(|group| group.count).sum();
        histogram[0] = PIXELS - lit;

        for group in pixels {
            histogram[bin_of(group.brightness)] += group.count;
        }

        let mut stars = StarVisibility::new();
        stars.update(&histogram);
        stars.visibility()
    }

    #[test]
    fn empty_space_shows_every_star() {
        assert_eq!(visibility(&[]), 1.0);
    }

    #[test]
    fn a_sunlit_planet_in_view_leaves_most_stars() {
        let visibility = visibility(&[Pixels {
            brightness: SUNLIT_EARTH,
            count: PIXELS / 3,
        }]);
        assert!(visibility > 0.9, "{visibility}");
    }

    #[test]
    fn a_full_moon_filling_the_view_dims_the_stars() {
        let visibility = visibility(&[Pixels {
            brightness: FULL_MOON,
            count: PIXELS / 2,
        }]);
        assert!((0.1..0.5).contains(&visibility), "{visibility}");
    }

    #[test]
    fn the_sun_in_view_hides_every_star() {
        assert_eq!(
            visibility(&[Pixels {
                brightness: SUN_DISK,
                count: SUN_DISK_PIXELS,
            }]),
            0.0
        );
    }

    #[test]
    fn each_fade_closes_half_the_gap_in_its_half_life() {
        let stars = StarVisibility::new();
        let hiding = stars.adapt(1.0, 0.0, STARS_FADE_OUT_HALF_LIFE_SECONDS);
        let revealing = stars.adapt(0.0, 1.0, STARS_FADE_IN_HALF_LIFE_SECONDS);
        assert!((hiding - 0.5).abs() < 1e-6 && (revealing - 0.5).abs() < 1e-6);
    }

    #[test]
    fn a_zero_duration_scale_adapts_at_once() {
        let mut stars = StarVisibility::new();
        stars.configure(0.0);
        assert_eq!(stars.adapt(0.0, 1.0, 0.0), 1.0);
    }
}
