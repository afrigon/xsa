use ash::vk;
use glam::{Mat4, Vec2};

// Halton (2, 3) points 1 to 8: evenly spread over the pixel, and short enough to converge quickly.
const JITTER_SAMPLE_COUNT: u32 = 8;
const JITTER_BASE_X: u32 = 2;
const JITTER_BASE_Y: u32 = 3;
const PIXEL_CENTER: f32 = 0.5;
const NDC_PER_VIEWPORT: f32 = 2.0;
pub(super) const HISTORY_IMAGE_COUNT: usize = 2;

// What temporal effects carry from one frame to the next: the previous transforms motion vectors are measured
// against, the jitter sequence and which history image holds the accumulated result.
#[derive(Default)]
pub(super) struct TemporalHistory {
    frame: u32,
    history_written: bool,
    history_valid: bool,
    view_projection: Option<Mat4>,
    exposure: Option<f32>,
    world_from_model: Vec<Mat4>,
    previous_view_projection: Mat4,
    previous_exposure: f32,
    previous_world_from_model: Vec<Mat4>,
}

impl TemporalHistory {
    pub fn begin_frame(&mut self, view_projection: Mat4, exposure: f32) {
        self.frame = self.frame.wrapping_add(1);
        self.history_valid = self.history_written;
        self.history_written = true;
        self.previous_view_projection = self.view_projection.replace(view_projection).unwrap_or(view_projection);
        self.previous_exposure = self.exposure.replace(exposure).unwrap_or(exposure);
        std::mem::swap(&mut self.world_from_model, &mut self.previous_world_from_model);
        self.world_from_model.clear();
    }

    // Objects are recorded in scene order; one that did not exist last frame counts as unmoved.
    pub fn record_object(&mut self, world_from_model: Mat4) -> Mat4 {
        let previous = self
            .previous_world_from_model
            .get(self.world_from_model.len())
            .copied()
            .unwrap_or(world_from_model);
        self.world_from_model.push(world_from_model);

        self.previous_view_projection * previous
    }

    pub fn reset(&mut self) {
        self.history_written = false;
    }

    // Normalized device coordinates to add to clip-space positions, within half a pixel of the center.
    pub fn jitter(&self, extent: vk::Extent2D) -> Vec2 {
        let sample = self.frame % JITTER_SAMPLE_COUNT + 1;
        let offset = Vec2::new(halton(sample, JITTER_BASE_X), halton(sample, JITTER_BASE_Y)) - PIXEL_CENTER;

        offset * NDC_PER_VIEWPORT / Vec2::new(extent.width as f32, extent.height as f32)
    }

    pub fn current(&self) -> usize {
        self.frame as usize % HISTORY_IMAGE_COUNT
    }

    pub fn previous(&self) -> usize {
        (self.frame as usize + 1) % HISTORY_IMAGE_COUNT
    }

    pub fn is_valid(&self) -> bool {
        self.history_valid
    }

    pub fn previous_view_projection(&self) -> Mat4 {
        self.previous_view_projection
    }

    // Colors are stored pre-exposed, so history from a frame with another exposure is rescaled to this one's.
    pub fn history_exposure_scale(&self) -> f32 {
        self.exposure.map_or(1.0, |exposure| exposure / self.previous_exposure)
    }
}

fn halton(mut index: u32, base: u32) -> f32 {
    let mut fraction = 1.0;
    let mut result = 0.0;

    while index > 0 {
        fraction /= base as f32;
        result += fraction * (index % base) as f32;
        index /= base;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halton_matches_the_radical_inverse() {
        assert_eq!(halton(1, 2), 0.5);
        assert_eq!(halton(2, 2), 0.25);
        assert_eq!(halton(3, 2), 0.75);
        assert!((halton(1, 3) - 1.0 / 3.0).abs() < 1e-6);
        assert!((halton(5, 3) - 7.0 / 9.0).abs() < 1e-6);
    }

    #[test]
    fn jitter_stays_within_half_a_pixel() {
        let extent = vk::Extent2D { width: 100, height: 50 };
        let mut history = TemporalHistory::default();

        for _ in 0..JITTER_SAMPLE_COUNT {
            history.begin_frame(Mat4::IDENTITY, 1.0);
            let pixels = history.jitter(extent) / NDC_PER_VIEWPORT * Vec2::new(100.0, 50.0);
            assert!(pixels.abs().max_element() <= 0.5);
        }
    }

    #[test]
    fn history_is_valid_from_the_second_frame_after_a_reset() {
        let mut history = TemporalHistory::default();
        history.begin_frame(Mat4::IDENTITY, 1.0);
        assert!(!history.is_valid());
        history.begin_frame(Mat4::IDENTITY, 1.0);
        assert!(history.is_valid());
        history.reset();
        history.begin_frame(Mat4::IDENTITY, 1.0);
        assert!(!history.is_valid());
        assert_ne!(history.current(), history.previous());
    }
}
