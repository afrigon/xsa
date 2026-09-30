use glam::{DQuat, DVec3, Vec3};

use super::Material;

#[derive(Clone, Copy)]
pub struct ObjectHandle(usize);

pub struct SceneObject {
    pub position: DVec3,
    pub orientation: DQuat,
    pub scale: f64,
    pub color: Vec3,
    pub material: Material,
}

#[derive(Default)]
pub struct Scene {
    objects: Vec<SceneObject>,
    pub sun_position: DVec3,
}

impl Scene {
    pub fn add(&mut self, object: SceneObject) -> ObjectHandle {
        self.objects.push(object);
        ObjectHandle(self.objects.len() - 1)
    }

    pub fn object_mut(&mut self, handle: ObjectHandle) -> &mut SceneObject {
        &mut self.objects[handle.0]
    }

    pub fn objects(&self) -> &[SceneObject] {
        &self.objects
    }
}
