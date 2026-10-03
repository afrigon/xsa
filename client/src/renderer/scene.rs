mod material_handle;
mod object_handle;
mod scene_object;

pub use material_handle::MaterialHandle;
pub use object_handle::ObjectHandle;
pub use scene_object::SceneObject;

use glam::{DVec3, Vec3};

use super::Material;

#[derive(Default)]
pub struct Scene {
    objects: Vec<SceneObject>,
    materials: Vec<Material>,
    pub sun_position: DVec3,
    pub sun_intensity: Vec3,
    pub skybox: Option<MaterialHandle>,
}

impl Scene {
    pub fn add(&mut self, object: SceneObject) -> ObjectHandle {
        self.objects.push(object);
        ObjectHandle {
            index: self.objects.len() - 1,
        }
    }

    pub fn object_mut(&mut self, handle: ObjectHandle) -> &mut SceneObject {
        &mut self.objects[handle.index]
    }

    pub fn objects(&self) -> &[SceneObject] {
        &self.objects
    }

    pub fn add_material(&mut self, material: Material) -> MaterialHandle {
        self.materials.push(material);
        MaterialHandle {
            index: self.materials.len() - 1,
        }
    }

    pub fn material(&self, handle: MaterialHandle) -> &Material {
        &self.materials[handle.index]
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }
}
