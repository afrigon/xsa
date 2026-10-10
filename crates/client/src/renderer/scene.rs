mod material_handle;
mod object_handle;
mod scene_object;
mod star;

pub use material_handle::MaterialHandle;
pub use object_handle::ObjectHandle;
pub use scene_object::SceneObject;
pub use star::Star;

use super::Material;

#[derive(Default)]
pub struct Scene {
    objects: Vec<SceneObject>,
    materials: Vec<Material>,
    stars: Vec<Star>,
    pub skybox: Option<MaterialHandle>,
}

impl Scene {
    pub fn add(&mut self, object: SceneObject) -> ObjectHandle {
        self.objects.push(object);
        ObjectHandle {
            index: self.objects.len() - 1,
        }
    }

    pub fn object(&self, handle: ObjectHandle) -> &SceneObject {
        &self.objects[handle.index]
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

    pub fn add_star(&mut self, star: Star) {
        self.stars.push(star);
    }

    pub fn stars(&self) -> &[Star] {
        &self.stars
    }
}
