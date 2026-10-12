use glam::{DQuat, DVec3};

pub struct SceneObject {
    pub position: DVec3,
    pub orientation: DQuat,
    pub scale: f64,
    pub bounding_radius: f64,
}
