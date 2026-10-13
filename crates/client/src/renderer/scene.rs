mod light_source;
mod light_source_handle;
mod material_handle;
mod object_handle;
mod scene_object;
mod star;
mod terrain;
mod terrain_handle;
mod terrain_height;

pub use light_source::LightSource;
pub use light_source_handle::LightSourceHandle;
pub use material_handle::MaterialHandle;
pub use object_handle::ObjectHandle;
pub use scene_object::SceneObject;
pub use star::Star;
pub use terrain::Terrain;
pub use terrain_handle::TerrainHandle;
pub use terrain_height::TerrainHeight;

#[derive(Default)]
pub struct Scene {
    objects: Vec<SceneObject>,
    light_sources: Vec<LightSource>,
    stars: Vec<Star>,
    terrains: Vec<Terrain>,
    occluders: Vec<ObjectHandle>,
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

    pub fn add_light_source(&mut self, light: LightSource) -> LightSourceHandle {
        self.light_sources.push(light);
        LightSourceHandle {
            index: self.light_sources.len() - 1,
        }
    }

    pub fn light_source(&self, handle: LightSourceHandle) -> &LightSource {
        &self.light_sources[handle.index]
    }

    pub fn light_sources(&self) -> &[LightSource] {
        &self.light_sources
    }

    pub fn add_star(&mut self, star: Star) {
        self.stars.push(star);
    }

    pub fn stars(&self) -> &[Star] {
        &self.stars
    }

    pub fn add_terrain(&mut self, terrain: Terrain) -> TerrainHandle {
        self.terrains.push(terrain);
        TerrainHandle {
            index: self.terrains.len() - 1,
        }
    }

    pub fn terrain(&self, handle: TerrainHandle) -> &Terrain {
        &self.terrains[handle.index]
    }

    pub fn terrains(&self) -> &[Terrain] {
        &self.terrains
    }

    // Objects whose sphere (of radius `scale`) hides the stars behind it.
    pub fn add_occluder(&mut self, object: ObjectHandle) {
        self.occluders.push(object);
    }

    pub fn occluders(&self) -> &[ObjectHandle] {
        &self.occluders
    }
}
