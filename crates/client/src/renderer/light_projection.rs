use ash::vk;
use glam::{DMat4, DVec2, DVec3, Vec3};

use super::gpu_data::ProjectedLightData;
use crate::camera::Camera;

pub(in crate::renderer) struct LightProjection {
    view_projection: DMat4,
    extent: DVec2,
    focal_length_pixels: f64,
}

impl LightProjection {
    pub fn new(camera: &Camera, extent: vk::Extent2D) -> LightProjection {
        let clip_from_view = camera.clip_from_view().as_dmat4();
        let height = f64::from(extent.height);

        LightProjection {
            view_projection: clip_from_view * camera.view_rotation().as_dmat4(),
            extent: DVec2::new(f64::from(extent.width), height),
            // Negative in Vulkan projections, which flip Y.
            focal_length_pixels: clip_from_view.y_axis.y.abs() * height / 2.0,
        }
    }

    pub fn focal_length_pixels(&self) -> f32 {
        self.focal_length_pixels as f32
    }

    // `position` is relative to the camera; `None` when the light is behind the camera or surrounds it.
    pub fn project(
        &self,
        position: DVec3,
        radius: f64,
        intensity: Vec3,
        visibility: f32,
    ) -> Option<ProjectedLightData> {
        let distance = position.length();
        let clip = self.view_projection * position.extend(1.0);

        if clip.w <= 0.0 || distance <= radius {
            return None;
        }

        let normalized = clip.truncate().truncate() / clip.w;
        let angular_radius = (radius / distance).asin();

        Some(ProjectedLightData {
            screen_position: ((normalized * 0.5 + 0.5) * self.extent).as_vec2(),
            visibility,
            angular_radius: angular_radius as f32,
            illuminance: intensity / (distance * distance) as f32,
            padding: 0,
        })
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
    const SUN_RADIUS: f64 = 6.957e8;
    const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;

    fn projection() -> LightProjection {
        let aspect_ratio = EXTENT.width as f32 / EXTENT.height as f32;

        LightProjection::new(&Camera::new(FRAC_PI_2, aspect_ratio), EXTENT)
    }

    #[test]
    fn a_star_straight_ahead_lands_on_the_screen_center() {
        let star = projection()
            .project(DVec3::new(0.0, ASTRONOMICAL_UNIT, 0.0), SUN_RADIUS, Vec3::ONE, 1.0)
            .unwrap();

        assert!((star.screen_position.x - 960.0).abs() < 0.5, "{}", star.screen_position);
        assert!((star.screen_position.y - 540.0).abs() < 0.5, "{}", star.screen_position);
    }

    #[test]
    fn the_sun_from_earth_is_about_a_quarter_degree_wide() {
        let star = projection()
            .project(DVec3::new(0.0, ASTRONOMICAL_UNIT, 0.0), SUN_RADIUS, Vec3::ONE, 1.0)
            .unwrap();
        let degrees = star.angular_radius.to_degrees();

        assert!((0.26..0.27).contains(&degrees), "{degrees}°");
    }

    #[test]
    fn a_star_behind_the_camera_is_not_projected() {
        assert!(
            projection()
                .project(DVec3::new(0.0, -ASTRONOMICAL_UNIT, 0.0), SUN_RADIUS, Vec3::ONE, 1.0)
                .is_none()
        );
    }
}
