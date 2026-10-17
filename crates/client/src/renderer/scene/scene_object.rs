use glam::{DQuat, DVec3};

pub struct SceneObject {
    pub position: DVec3,
    pub orientation: DQuat,
    pub scale: f64,
    // Its extent without its terrain: a terrain adds its highest point once loaded.
    pub bounding_radius: f64,
}
