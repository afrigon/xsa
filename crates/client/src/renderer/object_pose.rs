use glam::{DQuat, DVec3, Mat4, Vec3};

#[derive(Clone, Copy)]
pub(super) struct ObjectPose {
    pub position: DVec3,
    pub orientation: DQuat,
    pub scale: f64,
}

impl ObjectPose {
    pub fn world_from_model(&self, camera_position: DVec3) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::splat(self.scale as f32),
            self.orientation.as_quat(),
            (self.position - camera_position).as_vec3(),
        )
    }
}
