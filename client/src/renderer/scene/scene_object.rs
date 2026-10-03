use glam::{DQuat, DVec3};

use super::MaterialHandle;

pub struct SceneObject {
    pub position: DVec3,
    pub orientation: DQuat,
    pub scale: f64,
    pub material: MaterialHandle,
}
