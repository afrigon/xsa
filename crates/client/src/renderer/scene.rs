mod body_scatter;
mod light_source;
mod light_source_handle;
mod material_handle;
mod model;
mod model_handle;
mod node_material;
mod object_handle;
mod scene_object;
mod star;
mod terrain;
mod terrain_handle;

pub use body_scatter::BodyScatter;
pub use light_source::LightSource;
pub use light_source_handle::LightSourceHandle;
pub use material_handle::MaterialHandle;
pub use model::Model;
pub use model_handle::ModelHandle;
pub use node_material::NodeMaterial;
pub use object_handle::ObjectHandle;
pub use scene_object::SceneObject;
pub use star::Star;
pub use terrain::Terrain;
pub use terrain_handle::TerrainHandle;

#[derive(Default)]
pub struct Scene {
    objects: Vec<SceneObject>,
    light_sources: Vec<LightSource>,
    stars: Vec<Star>,
    terrains: Vec<Terrain>,
    occluders: Vec<ObjectHandle>,
    models: Vec<Model>,
    scatters: Vec<BodyScatter>,
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

    pub fn add_model(&mut self, model: Model) -> ModelHandle {
        self.models.push(model);
        ModelHandle {
            index: self.models.len() - 1,
        }
    }

    pub fn model(&self, handle: ModelHandle) -> &Model {
        &self.models[handle.index]
    }

    pub fn models(&self) -> &[Model] {
        &self.models
    }

    pub fn add_scatter(&mut self, scatter: BodyScatter) {
        self.scatters.push(scatter);
    }

    pub fn scatters(&self) -> &[BodyScatter] {
        &self.scatters
    }

    // Objects whose sphere (of radius `scale`) hides the stars behind it.
    pub fn add_occluder(&mut self, object: ObjectHandle) {
        self.occluders.push(object);
    }

    pub fn occluders(&self) -> &[ObjectHandle] {
        &self.occluders
    }
}
