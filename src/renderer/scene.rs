use glam::{DQuat, DVec3};

use super::Material;

#[derive(Clone, Copy)]
pub struct ObjectHandle(usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MaterialHandle(usize);

impl MaterialHandle {
    pub fn index(self) -> usize {
        self.0
    }
}

pub struct SceneObject {
    pub position: DVec3,
    pub orientation: DQuat,
    pub scale: f64,
    pub material: MaterialHandle,
}

#[derive(Default)]
pub struct Scene {
    objects: Vec<SceneObject>,
    materials: Vec<Material>,
    pub sun_position: DVec3,
    pub skybox: Option<MaterialHandle>,
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

    pub fn add_material(&mut self, material: Material) -> MaterialHandle {
        self.materials.push(material);
        MaterialHandle(self.materials.len() - 1)
    }

    pub fn material(&self, handle: MaterialHandle) -> &Material {
        &self.materials[handle.0]
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }
}
