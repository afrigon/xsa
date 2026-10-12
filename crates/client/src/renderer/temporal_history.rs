use ash::vk;
use glam::{DVec3, Mat4, Vec2};

use super::object_pose::ObjectPose;

// Halton (2, 3) points 1 to 8: evenly spread over the pixel, and short enough to converge quickly.
const JITTER_SAMPLE_COUNT: u32 = 8;
const JITTER_BASE_X: u32 = 2;
const JITTER_BASE_Y: u32 = 3;
const PIXEL_CENTER: f32 = 0.5;
const NDC_PER_VIEWPORT: f32 = 2.0;

// What temporal effects carry from one frame to the next: the previous transforms motion vectors are measured
// against, the jitter sequence and whether the accumulated history is usable.
#[derive(Default)]
pub(super) struct TemporalHistory {
    frame: u32,
    history_written: bool,
    history_valid: bool,
    view_projection: Option<Mat4>,
    exposure: Option<f32>,
    camera_position: Option<DVec3>,
    poses: Vec<ObjectPose>,
    previous_view_projection: Mat4,
    previous_exposure: f32,
    previous_camera_position: DVec3,
    previous_poses: Vec<ObjectPose>,
}

impl TemporalHistory {
    pub fn begin_frame(&mut self, view_projection: Mat4, exposure: f32, camera_position: DVec3) {
        self.frame = self.frame.wrapping_add(1);
        self.history_valid = self.history_written;
        self.history_written = true;
        self.previous_view_projection = self.view_projection.replace(view_projection).unwrap_or(view_projection);
        self.previous_exposure = self.exposure.replace(exposure).unwrap_or(exposure);
        self.previous_camera_position = self.camera_position.replace(camera_position).unwrap_or(camera_position);
        std::mem::swap(&mut self.poses, &mut self.previous_poses);
        self.poses.clear();
    }

    // Objects are recorded in scene order; one that did not exist last frame counts as unmoved.
    pub fn record_object(&mut self, pose: ObjectPose) -> ObjectPose {
        self.poses.push(pose);

        self.previous_pose(self.poses.len() - 1)
    }

    pub fn previous_pose(&self, object_index: usize) -> ObjectPose {
        self.previous_poses
            .get(object_index)
            .copied()
            .unwrap_or(self.poses[object_index])
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

    pub fn is_valid(&self) -> bool {
        self.history_valid
    }

    pub fn previous_view_projection(&self) -> Mat4 {
        self.previous_view_projection
    }

    pub fn previous_camera_position(&self) -> DVec3 {
        self.previous_camera_position
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
            history.begin_frame(Mat4::IDENTITY, 1.0, DVec3::ZERO);
            let pixels = history.jitter(extent) / NDC_PER_VIEWPORT * Vec2::new(100.0, 50.0);
            assert!(pixels.abs().max_element() <= 0.5);
        }
    }

    #[test]
    fn history_is_valid_from_the_second_frame_after_a_reset() {
        let mut history = TemporalHistory::default();
        history.begin_frame(Mat4::IDENTITY, 1.0, DVec3::ZERO);
        assert!(!history.is_valid());
        history.begin_frame(Mat4::IDENTITY, 1.0, DVec3::ZERO);
        assert!(history.is_valid());
        history.reset();
        history.begin_frame(Mat4::IDENTITY, 1.0, DVec3::ZERO);
        assert!(!history.is_valid());
    }
}
