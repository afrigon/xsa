use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use glam::Vec3;
use resource_vulkan::{LoadRequest, ResourceGroup, Resources};
use xsa_core::scatter::ScatterPlacement;
use xsa_core::simulation::Body;
use xsa_packs::{
    BiomeDefinition, BiomeScatter, Id, MeshMaterialDefinition, ModelDefinition, PackStack, ScatterDefinition,
};

use crate::renderer::{
    BodyScatter, HapkeMaterial, MaterialHandle, Model, ModelHandle, NodeMaterial, Renderer, TerrainHandle,
};

// Loads the scatters bodies' biomes place, each model and mesh material once however many scatters share it.
pub(super) struct ScatterLoader<'a> {
    pub stack: &'a PackStack,
    pub resources: &'a Resources,
    pub group: &'a mut ResourceGroup,
    models: HashMap<Id, ModelHandle>,
    materials: HashMap<Id, MaterialHandle>,
}

impl<'a> ScatterLoader<'a> {
    pub fn new(stack: &'a PackStack, resources: &'a Resources, group: &'a mut ResourceGroup) -> ScatterLoader<'a> {
        ScatterLoader {
            stack,
            resources,
            group,
            models: HashMap::new(),
            materials: HashMap::new(),
        }
    }

    // Each scatter is seeded by its place in the biome's list, so a biome can place the same scatter twice. A scatter
    // whose models cannot load is skipped with a warning, so content loads while its files are missing.
    pub fn load_body(
        &mut self,
        renderer: &mut Renderer,
        body: &Body,
        terrain: TerrainHandle,
        biome: &Id,
        ground_albedo: Vec3,
    ) -> anyhow::Result<()> {
        let definition = self.stack.load_data::<BiomeDefinition>(biome)?;

        for (place, scatter) in definition.scatters.iter().enumerate() {
            match self.load_scatter(renderer, body, terrain, ground_albedo, place, scatter) {
                Ok(scatter) => renderer.scene_mut().add_scatter(scatter),
                Err(err) => tracing::warn!("{}: skipping scatter {}: {err:#}", body.id, scatter.scatter),
            }
        }

        Ok(())
    }

    fn load_scatter(
        &mut self,
        renderer: &mut Renderer,
        body: &Body,
        terrain: TerrainHandle,
        ground_albedo: Vec3,
        place: usize,
        scatter: &BiomeScatter,
    ) -> anyhow::Result<BodyScatter> {
        let definition = self.stack.load_data::<ScatterDefinition>(&scatter.scatter)?;
        let models = definition
            .models
            .iter()
            .map(|model| self.load_model(renderer, &model.model))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let seed = body.seed.derive(&format!("scatter/{place}"));

        Ok(BodyScatter {
            terrain,
            placement: Arc::new(ScatterPlacement::new(
                definition.scatter(),
                scatter.density,
                seed,
                body.radius,
            )),
            models,
            tint: scatter.tint,
            ground_albedo,
        })
    }

    fn load_model(&mut self, renderer: &mut Renderer, id: &Id) -> anyhow::Result<ModelHandle> {
        if let Some(&model) = self.models.get(id) {
            return Ok(model);
        }

        let definition = ModelDefinition::load(self.stack, id).with_context(|| format!("model {id}"))?;
        let mesh = self
            .resources
            .load(LoadRequest::new(&definition.mesh, format!("{id} mesh")), ());
        self.group.add(&mesh);
        let materials = definition
            .materials
            .iter()
            .map(|material| {
                Ok(NodeMaterial {
                    node: material.node.clone(),
                    material: self.load_material(renderer, &material.material)?,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let model = renderer.scene_mut().add_model(Model {
            name: id.to_string(),
            mesh,
            materials,
        });
        self.models.insert(id.clone(), model);

        Ok(model)
    }

    fn load_material(&mut self, renderer: &mut Renderer, id: &Id) -> anyhow::Result<MaterialHandle> {
        if let Some(&material) = self.materials.get(id) {
            return Ok(material);
        }

        let definition = MeshMaterialDefinition::load(self.stack, id).with_context(|| format!("material {id}"))?;
        let material = match definition {
            MeshMaterialDefinition::Hapke {
                base_color,
                lobe_width,
                lobe_balance,
                macroscopic_roughness_degrees,
                porosity,
                shadow_hiding_amplitude,
                shadow_hiding_width,
                backscatter_amplitude,
                backscatter_width,
            } => renderer.add_material(
                &format!("{id} material"),
                HapkeMaterial {
                    base_color,
                    lobe_width,
                    lobe_balance,
                    macroscopic_roughness: macroscopic_roughness_degrees.to_radians(),
                    porosity,
                    shadow_hiding_amplitude,
                    shadow_hiding_width,
                    backscatter_amplitude,
                    backscatter_width,
                },
            )?,
        };
        self.materials.insert(id.clone(), material);

        Ok(material)
    }
}
