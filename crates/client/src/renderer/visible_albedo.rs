// Must match histogram.slang.
pub(in crate::renderer) const ALBEDO_HISTOGRAM_BINS: usize = 64;
const MINIMUM_LOG2_ALBEDO: f32 = -8.0;
const MAXIMUM_LOG2_ALBEDO: f32 = 2.0;

// The apparent albedo of the lit surfaces in view: how bright they look against a white diffuser in the light they
// receive. Unlit pixels (space, night sides, stars) are not counted.
pub struct VisibleAlbedo {
    albedo: Option<f32>,
}

impl VisibleAlbedo {
    pub fn new() -> Self {
        Self { albedo: None }
    }

    // The log-average over the pixels binned by histogram.slang; None when no lit surface is in view.
    pub fn update(&mut self, histogram: &[u32]) {
        let bin_width = (MAXIMUM_LOG2_ALBEDO - MINIMUM_LOG2_ALBEDO) / histogram.len() as f32;
        let (count, log2_sum) = histogram
            .iter()
            .enumerate()
            .fold((0.0, 0.0), |(count, sum), (bin, &pixels)| {
                let center = MINIMUM_LOG2_ALBEDO + (bin as f32 + 0.5) * bin_width;
                (count + pixels as f64, sum + pixels as f64 * center as f64)
            });
        self.albedo = (count > 0.0).then(|| ((log2_sum / count) as f32).exp2());
    }

    pub fn albedo(&self) -> Option<f32> {
        self.albedo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn histogram_of(albedo: f32) -> Vec<u32> {
        let mut histogram = vec![0; ALBEDO_HISTOGRAM_BINS];
        let position = (albedo.log2() - MINIMUM_LOG2_ALBEDO) / (MAXIMUM_LOG2_ALBEDO - MINIMUM_LOG2_ALBEDO);
        histogram[(position * ALBEDO_HISTOGRAM_BINS as f32) as usize] = 1000;
        histogram
    }

    #[test]
    fn nothing_lit_in_view_has_no_albedo() {
        let mut visible = VisibleAlbedo::new();
        visible.update(&vec![0; ALBEDO_HISTOGRAM_BINS]);
        assert_eq!(visible.albedo(), None);
    }

    #[test]
    fn a_uniform_surface_reads_its_own_albedo() {
        let mut visible = VisibleAlbedo::new();
        visible.update(&histogram_of(0.11));
        let albedo = visible.albedo().unwrap();
        assert!((albedo / 0.11).log2().abs() < 0.1, "{albedo}");
    }
}
