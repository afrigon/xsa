use glam::{DMat3, DQuat, DVec3};

use super::Camera;
use super::orbit::OrbitTarget;

const ANGULAR_RADIUS_DEGREES: f64 = 22.0;
const AZIMUTH_FROM_STAR_DEGREES: f64 = 35.0;
const ELEVATION_DEGREES: f64 = 15.0;
const HORIZONTAL_SCREEN_OFFSET: f64 = 0.36;
const UNLIT_VIEW_DIRECTION: DVec3 = DVec3::NEG_Y;

// Frames a body large on the right of the screen, seen from its lit side a little off the star's direction so the
// terminator shows.
pub struct MenuCamera;

impl MenuCamera {
    pub fn update(&self, camera: &mut Camera, target: OrbitTarget, star_position: Option<DVec3>) {
        let star_direction = star_position.map_or(UNLIT_VIEW_DIRECTION, |star| star - target.position);
        let heading = DVec3::new(star_direction.x, star_direction.y, 0.0).normalize_or(UNLIT_VIEW_DIRECTION);
        let heading = DQuat::from_rotation_z(AZIMUTH_FROM_STAR_DEGREES.to_radians()) * heading;
        let elevation = ELEVATION_DEGREES.to_radians();
        let view_direction = heading * elevation.cos() + DVec3::Z * elevation.sin();
        let distance = target.radius / ANGULAR_RADIUS_DEGREES.to_radians().sin();

        let forward = -view_direction;
        let right = forward.cross(DVec3::Z).normalize();
        let up = right.cross(forward);
        let looking_at_body = DQuat::from_mat3(&DMat3::from_cols(right, forward, up));
        let half_field_of_view = f64::from(camera.horizontal_fov) / 2.0;
        let offset_angle = (HORIZONTAL_SCREEN_OFFSET * half_field_of_view.tan()).atan();

        camera.position = target.position + view_direction * distance;
        camera.orientation = looking_at_body * DQuat::from_rotation_z(offset_angle);
    }
}
