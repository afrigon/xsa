use ash::vk;
use glam::DVec3;

use super::frustum::Frustum;
use crate::camera::Camera;

const MINIMUM_PROJECTED_DIAMETER_PIXELS: f64 = 1.0;

pub(in crate::renderer) struct Culler {
    frustum: Frustum,
    projected_diameter_scale: f64,
}

impl Culler {
    pub fn new(camera: &Camera, extent: vk::Extent2D) -> Self {
        let clip_from_view = camera.clip_from_view();
        let view_projection = clip_from_view * camera.view_rotation();

        Self {
            frustum: Frustum::from_view_projection(view_projection.as_dmat4()),
            // Negative in Vulkan projections, which flip Y.
            projected_diameter_scale: f64::from(clip_from_view.y_axis.y.abs()) * f64::from(extent.height),
        }
    }

    pub fn is_visible(&self, center: DVec3, radius: f64) -> bool {
        radius / center.length() * self.projected_diameter_scale >= MINIMUM_PROJECTED_DIAMETER_PIXELS
            && self.frustum.contains_sphere(center, radius)
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;

    const EXTENT: vk::Extent2D = vk::Extent2D {
        width: 1920,
        height: 1080,
    };

    fn culler() -> Culler {
        let aspect_ratio = EXTENT.width as f32 / EXTENT.height as f32;

        Culler::new(&Camera::new(FRAC_PI_2, aspect_ratio), EXTENT)
    }

    #[test]
    fn sphere_several_pixels_wide_is_visible() {
        assert!(culler().is_visible(DVec3::new(0.0, 100.0, 0.0), 1.0));
    }

    #[test]
    fn sphere_under_a_pixel_wide_is_culled() {
        assert!(!culler().is_visible(DVec3::new(0.0, 10_000.0, 0.0), 1.0));
    }

    #[test]
    fn sphere_outside_the_frustum_is_culled() {
        assert!(!culler().is_visible(DVec3::new(0.0, -100.0, 0.0), 1.0));
    }

    #[test]
    fn sphere_surrounding_the_camera_is_visible() {
        assert!(culler().is_visible(DVec3::new(0.0, 1.0, 0.0), 10.0));
    }
}
