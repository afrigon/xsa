use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, bail, ensure};
use glam::{DQuat, DVec3, Vec3};
use resource_vulkan::{GroupProgress, LoadPriority, LoadRequest, ResourceGroup, Resources};
use xsa_commands::completion::CompletionCandidate;
use xsa_core::daylight::{Daylight, SkyPosition};
use xsa_core::light::{Illumination, StarLight};
use xsa_core::simulation::{Body, BodyCategory, BodyId, BodyIndex, BodyState, Simulation, SimulationState};
use xsa_packs::{
    BiomeSurfacesDefinition, Id, LensDefinition, MaterialDefinition, PackStack, SimulationDefinition, SkyboxDefinition,
    SurfaceDefinition, TerrainDefinition,
};
use xsa_proto::world::WorldState;
use xsa_units::SimulationTime;

use super::body_step::BodyStep;
use crate::camera::Attitude;
use crate::renderer::{
    BiomeSurfaces, ColorSpace, EmissiveMaterial, HapkeMaterial, LambertMaterial, LightSource, MaterialHandle,
    ObjectHandle, Renderer, SceneObject, SkyboxMaterial, Star, SurfaceClass, SurfaceTextures, Terrain, TerrainLayer,
    TerrainMaps,
};
use crate::ui::{PositionReading, SurfacePosition};

const DEFAULT_NAMESPACE: &str = "base";
const MAIN_LENS: &str = "main";
// Altitudes are read at the feet of a standing person, this far below the eyes.
const EYE_HEIGHT_METERS: f64 = 1.8;

pub(crate) struct ClientWorld {
    simulation_id: Id,
    simulation: Simulation,
    state: SimulationState,
    // The simulation time `state` is at.
    time: SimulationTime,
    illumination: Illumination,
    body_objects: Vec<ObjectHandle>,
    // One per body; None for a smooth body without layers. Its ground is the sea until its heights load.
    terrains: Vec<Option<Arc<TerrainMaps>>>,
    skybox: Option<MaterialHandle>,
    // Every file the scene needs, for its progress.
    resources: ResourceGroup,
    priority: LoadPriority,
}

impl ClientWorld {
    // Builds the scene at once; its files load in the background and appear as they arrive.
    pub fn load(
        packs_directory: &Path,
        world: &WorldState,
        renderer: &mut Renderer,
        resources: &Resources,
    ) -> anyhow::Result<ClientWorld> {
        let pack_ids: Vec<String> = world.packs.iter().map(|pack| pack.id.clone()).collect();
        let stack = PackStack::load(packs_directory, &pack_ids)?;

        for (expected, installed) in world.packs.iter().zip(stack.manifests()) {
            ensure!(
                installed.version.to_string() == expected.version,
                "the server uses {} {}, but {} is installed",
                expected.id,
                expected.version,
                installed.version
            );
        }

        let mut group = ResourceGroup::new();
        let simulation_id = Id::parse(&world.simulation, DEFAULT_NAMESPACE)?;
        let simulation = stack.load_data::<SimulationDefinition>(&simulation_id)?.build(&stack)?;
        let illumination = Illumination::new(simulation.bodies());
        let mut materials = Vec::new();
        let mut terrains = Vec::new();
        let mut surfaces = Vec::new();

        for (value, body) in simulation.bodies().iter().enumerate() {
            let id = Id::parse(&body.id.value, DEFAULT_NAMESPACE)?;
            let definition = MaterialDefinition::load_for_body(&stack, &id)
                .with_context(|| format!("loading the material of {}", body.id))?;
            let terrain_definition = TerrainDefinition::load_for_body(&stack, &id)
                .with_context(|| format!("loading the terrain of {}", body.id))?;
            let terrain =
                ClientWorld::load_terrain(&id, terrain_definition.as_ref(), &definition, resources, &mut group);
            let body_surfaces = ClientWorld::load_surfaces(
                &stack,
                body,
                terrain_definition.as_ref(),
                &definition,
                resources,
                &mut group,
            )
            .with_context(|| format!("loading the surfaces of {}", body.id))?
            .map(Arc::new);
            let star_light = illumination.star_light(BodyIndex { value });
            let material = ClientWorld::create_material(
                renderer,
                resources,
                &mut group,
                &body.id,
                definition,
                body_surfaces.clone(),
                star_light,
            )
            .with_context(|| format!("loading the material of {}", body.id))?;
            materials.push(material);
            terrains.push(terrain.map(Arc::new));
            surfaces.push(body_surfaces);
        }

        let mut state = SimulationState::default();
        simulation.state_at(world.time, &mut state);

        let scene = renderer.scene_mut();
        let body_objects: Vec<ObjectHandle> = simulation
            .bodies()
            .iter()
            .zip(&materials)
            .zip(&terrains)
            .zip(&surfaces)
            .zip(&state.bodies)
            .map(|((((body, &material), terrain), surfaces), body_state)| {
                let object = scene.add(SceneObject {
                    position: body_state.position,
                    orientation: body_state.orientation,
                    scale: body.radius,
                    bounding_radius: body.radius,
                });
                scene.add_terrain(Terrain {
                    object,
                    material,
                    maps: terrain.clone(),
                    surfaces: surfaces.clone(),
                });
                scene.add_occluder(object);
                object
            })
            .collect();

        for (value, &object) in body_objects.iter().enumerate() {
            if let Some(star_light) = illumination.star_light(BodyIndex { value }) {
                let light = scene.add_light_source(LightSource {
                    object,
                    intensity: star_light.color * star_light.luminous_intensity as f32,
                });
                scene.add_star(Star { light });
            }
        }

        let skybox = ClientWorld::load_skybox(&stack, &simulation_id, renderer, resources, &mut group)?;
        renderer.scene_mut().skybox = skybox;
        ClientWorld::load_lens(&stack, renderer, resources, &mut group);

        Ok(ClientWorld {
            simulation_id,
            simulation,
            state,
            time: world.time,
            illumination,
            body_objects,
            terrains,
            skybox,
            resources: group,
            priority: LoadPriority::Normal,
        })
    }

    pub fn progress(&self) -> GroupProgress {
        self.resources.progress()
    }

    pub fn set_priority(&mut self, resources: &Resources, priority: LoadPriority) {
        if self.priority != priority {
            resources.set_priority(&self.resources, priority);
            self.priority = priority;
        }
    }

    pub fn skybox(&self) -> Option<MaterialHandle> {
        self.skybox
    }

    pub fn bodies(&self) -> &[Body] {
        self.simulation.bodies()
    }

    pub fn body(&self, index: BodyIndex) -> &Body {
        self.simulation.body(index)
    }

    pub fn body_state(&self, index: BodyIndex) -> &BodyState {
        self.state.body(index)
    }

    pub fn initial_target(&self) -> Option<BodyIndex> {
        self.simulation.spawn().or_else(|| {
            self.bodies()
                .iter()
                .position(|body| body.star.is_none())
                .or((!self.bodies().is_empty()).then_some(0))
                .map(|value| BodyIndex { value })
        })
    }

    pub fn find_body(&self, text: &str) -> anyhow::Result<BodyIndex> {
        let id = BodyId::from(&Id::parse(text, &self.simulation_id.namespace)?);

        self.simulation
            .find_body(&id)
            .with_context(|| format!("no body {id} in {}", self.simulation_id))
    }

    pub fn illuminance_at(&self, position: DVec3) -> f64 {
        self.illumination
            .illuminance_at(self.simulation.bodies(), &self.state.bodies, position)
    }

    // The camera against `index`: latitude, longitude and altitudes at the feet, EYE_HEIGHT_METERS below the camera,
    // and its attitude against the local horizon.
    pub fn position_reading(&self, index: BodyIndex, position: DVec3, orientation: DQuat) -> PositionReading {
        let body = self.body(index);
        let state = self.body_state(index);
        let offset = position - state.position;
        let above_sea = offset.length() - body.radius - EYE_HEIGHT_METERS;
        let surface = offset.try_normalize().map(|up| {
            let direction = state.orientation.inverse() * up;
            let latitude = direction.z.clamp(-1.0, 1.0).asin();
            let longitude = direction.y.atan2(direction.x);

            SurfacePosition {
                latitude,
                longitude,
                above_ground: self.terrain_height(index, direction).map(|height| above_sea - height),
                attitude: Attitude::of(orientation, up, self.local_north(index, latitude, longitude)),
            }
        });

        PositionReading {
            body: body.id.to_string(),
            above_sea,
            surface,
        }
    }

    // North along the ground at a latitude and longitude, in radians: along their meridian, so at a pole, where every
    // direction is south, the longitude still picks one.
    pub fn local_north(&self, index: BodyIndex, latitude: f64, longitude: f64) -> DVec3 {
        let north = DVec3::new(
            -latitude.sin() * longitude.cos(),
            -latitude.sin() * longitude.sin(),
            latitude.cos(),
        );

        self.body_state(index).orientation * north
    }

    // Meters relative to the radius at `direction`, a unit vector in the body's frame; 0 on a smooth body.
    pub fn ground_height(&self, index: BodyIndex, direction: DVec3) -> f64 {
        self.terrain_height(index, direction).unwrap_or(0.0)
    }

    // Whether the body's ground is as fine as the camera needs: its heights resident to the level the camera's distance
    // asks for, with nothing finer on its way.
    pub fn is_ground_settled(&self, index: BodyIndex) -> bool {
        self.terrains[index.value]
            .as_ref()
            .is_none_or(|terrain| terrain.is_ground_settled())
    }

    // Where the eyes are when the feet stand `altitude` above the ground at `direction`, in the body's frame.
    pub fn eye_position(&self, index: BodyIndex, direction: DVec3, altitude: f64) -> DVec3 {
        let state = self.body_state(index);
        let distance = self.body(index).radius + self.ground_height(index, direction) + altitude + EYE_HEIGHT_METERS;

        state.position + state.orientation * (direction * distance)
    }

    // None for a smooth body, and until the heights there load.
    fn terrain_height(&self, index: BodyIndex, direction: DVec3) -> Option<f64> {
        self.terrains[index.value].as_ref()?.height_at(direction)
    }

    pub fn body_candidates(&self) -> Vec<CompletionCandidate> {
        self.bodies()
            .iter()
            .map(|body| CompletionCandidate {
                value: body.id.to_string(),
                display: None,
                description: Some(body.category.name().to_string()),
                scope: None,
            })
            .collect()
    }

    // Wraps around the system; with a category, bodies of other categories are skipped.
    pub(super) fn step_body(
        &self,
        current: BodyIndex,
        step: BodyStep,
        category: Option<BodyCategory>,
    ) -> anyhow::Result<BodyIndex> {
        let count = self.bodies().len();
        let mut index = current;

        for _ in 0..count {
            index = match step {
                BodyStep::Next => index.next(count),
                BodyStep::Previous => index.previous(count),
            };

            if category.is_none_or(|category| self.body(index).category == category) {
                return Ok(index);
            }
        }

        let category = category.map_or("body".to_string(), |category| category.to_string());
        bail!("no {category} in {}", self.simulation_id)
    }

    pub fn advance(&mut self, time: SimulationTime) {
        self.simulation.state_at(time, &mut self.state);
        self.time = time;
    }

    pub fn time(&self) -> SimulationTime {
        self.time
    }

    pub fn daylight(&self, body: BodyIndex, star: BodyIndex) -> Daylight<'_> {
        Daylight::new(&self.simulation, body, star)
    }

    // Where `other` stands in `body`'s sky now.
    pub fn sky_position(&self, body: BodyIndex, other: BodyIndex) -> SkyPosition {
        SkyPosition::at(&self.simulation, body, other, self.time)
    }

    // The star named `id`, else the one lighting `body` most.
    pub fn star_for(&self, body: BodyIndex, id: Option<&str>) -> anyhow::Result<BodyIndex> {
        let Some(id) = id else {
            return self
                .brightest_star(self.body_state(body).position)
                .with_context(|| format!("no star lights {}", self.body(body).id));
        };
        let star = self.find_body(id)?;
        ensure!(self.body(star).star.is_some(), "{id} is not a star");

        Ok(star)
    }

    // The star that lights `position` most.
    pub fn brightest_star(&self, position: DVec3) -> Option<BodyIndex> {
        self.illumination.brightest_star(&self.state.bodies, position)
    }

    pub fn sync(&self, renderer: &mut Renderer) {
        let scene = renderer.scene_mut();

        for (state, &handle) in self.state.bodies.iter().zip(&self.body_objects) {
            let object = scene.object_mut(handle);
            object.position = state.position;
            object.orientation = state.orientation;
        }
    }

    fn load_lens(stack: &PackStack, renderer: &mut Renderer, resources: &Resources, group: &mut ResourceGroup) {
        let definition =
            match Id::parse(MAIN_LENS, DEFAULT_NAMESPACE).and_then(|profile| LensDefinition::load(stack, &profile)) {
                Ok(definition) => definition,
                Err(err) => {
                    tracing::warn!("skipping lens effects: lens {MAIN_LENS}: {err:#}");
                    return;
                }
            };
        let starburst = resources.load(LoadRequest::new(&definition.starburst.texture, "lens starburst"), ());
        group.add(&starburst);
        let ghosts = definition
            .ghosts
            .iter()
            .enumerate()
            .map(|(index, ghost)| {
                let ghost = resources.load(LoadRequest::new(&ghost.texture, format!("lens ghost {index}")), ());
                group.add(&ghost);
                ghost
            })
            .collect();
        renderer.load_lens(definition, starburst, ghosts);
    }

    // Drawn once its cube map loads.
    fn load_skybox(
        stack: &PackStack,
        simulation_id: &Id,
        renderer: &mut Renderer,
        resources: &Resources,
        group: &mut ResourceGroup,
    ) -> anyhow::Result<Option<MaterialHandle>> {
        let Some(skybox) = SkyboxDefinition::load(stack, simulation_id)? else {
            return Ok(None);
        };
        let cube_map = resources.load(LoadRequest::new(&skybox.texture, "skybox"), ColorSpace::Srgb);
        group.add(&cube_map);
        let material = renderer.add_material(
            "skybox",
            SkyboxMaterial {
                cube_map,
                orientation: skybox.orientation,
                luminance: skybox.luminance,
            },
        )?;

        Ok(Some(material))
    }

    // The body's heights and the layers its material shades with; None when it has neither, drawn smooth.
    fn load_terrain(
        body: &Id,
        terrain: Option<&TerrainDefinition>,
        material: &MaterialDefinition,
        resources: &Resources,
        group: &mut ResourceGroup,
    ) -> Option<TerrainMaps> {
        let heights = terrain.and_then(|definition| definition.height_map.as_ref());
        let layers = match material {
            MaterialDefinition::Hapke {
                color,
                normal,
                scattering,
                surge,
                ..
            } => TerrainLayer::ALL.map(|layer| match layer {
                TerrainLayer::Color => color.clone(),
                TerrainLayer::Normal => normal.clone(),
                TerrainLayer::Scattering => scattering.clone(),
                TerrainLayer::Surge => surge.clone(),
            }),
            MaterialDefinition::Lambert { color, normal, .. } => TerrainLayer::ALL.map(|layer| match layer {
                TerrainLayer::Color => color.clone(),
                TerrainLayer::Normal => normal.clone(),
                TerrainLayer::Scattering | TerrainLayer::Surge => None,
            }),
            MaterialDefinition::Emissive { .. } => Default::default(),
        };

        if heights.is_none() && layers.iter().all(Option::is_none) {
            return None;
        }

        Some(TerrainMaps::new(resources, group, &body.to_string(), heights, layers))
    }

    // The surfaces of the body's biome, for ground shaded with Hapke; None without a biome, and while a surface's
    // textures are missing.
    fn load_surfaces(
        stack: &PackStack,
        body: &Body,
        terrain: Option<&TerrainDefinition>,
        material: &MaterialDefinition,
        resources: &Resources,
        group: &mut ResourceGroup,
    ) -> anyhow::Result<Option<BiomeSurfaces>> {
        let MaterialDefinition::Hapke { .. } = material else {
            return Ok(None);
        };
        let Some(biome) = terrain.and_then(|definition| definition.biomes.first()) else {
            return Ok(None);
        };
        let Some(definition) = BiomeSurfacesDefinition::load(stack, biome)? else {
            return Ok(None);
        };
        let seed = body.seed.derive(&format!("biome/{biome}"));
        let definition_of = |class: SurfaceClass| match class {
            SurfaceClass::Flat => &definition.flat,
            SurfaceClass::Slope => &definition.slope,
        };
        let mut load = |class: SurfaceClass, surface: &SurfaceDefinition| {
            let mut array = |kind: &str, path: &PathBuf, color_space| {
                let name = format!("{} {} {kind}", body.id, class.name());
                let array = resources.load(LoadRequest::new(path, name), color_space);
                group.add(&array);
                array
            };
            SurfaceTextures::new(
                array("color", &surface.color, ColorSpace::Srgb),
                array("normal", &surface.normal, ColorSpace::Linear),
                array("displacement", &surface.displacement, ColorSpace::Linear),
                surface,
                class,
                seed,
            )
        };
        let [flat, slope] = SurfaceClass::ALL.map(|class| load(class, definition_of(class)));

        Ok(Some(BiomeSurfaces::new([flat?, slope?], &definition, seed)))
    }

    fn create_material(
        renderer: &mut Renderer,
        resources: &Resources,
        group: &mut ResourceGroup,
        body: &BodyId,
        definition: MaterialDefinition,
        surfaces: Option<Arc<BiomeSurfaces>>,
        star_light: Option<&StarLight>,
    ) -> anyhow::Result<MaterialHandle> {
        let name = format!("{body} material");

        let mut load = |kind: &str, path: &PathBuf, color_space| {
            let texture = resources.load(LoadRequest::new(path, format!("{body} {kind}")), color_space);
            group.add(&texture);
            texture
        };

        match definition {
            MaterialDefinition::Lambert {
                base_color,
                emissive,
                emissive_luminance,
                ..
            } => {
                let material = LambertMaterial {
                    base_color,
                    emissive: emissive.map(|path| load("emissive", &path, ColorSpace::Srgb)),
                    emissive_luminance,
                };
                renderer.add_material(&name, material)
            }
            MaterialDefinition::Hapke {
                base_color,
                emissive,
                emissive_luminance,
                macroscopic_roughness_degrees,
                porosity,
                ..
            } => {
                let material = HapkeMaterial {
                    base_color,
                    emissive: emissive.map(|path| load("emissive", &path, ColorSpace::Srgb)),
                    emissive_luminance,
                    macroscopic_roughness: macroscopic_roughness_degrees.to_radians(),
                    porosity,
                    surfaces,
                };
                renderer.add_material(&name, material)
            }
            MaterialDefinition::Emissive { color, luminance } => {
                let luminance = match star_light {
                    Some(star_light) => star_light.color * star_light.surface_luminance as f32,
                    None => Vec3::splat(luminance.context("an emissive material needs a `luminance` in cd/m²")?),
                };

                renderer.add_material(
                    &name,
                    EmissiveMaterial {
                        luminance: color * luminance,
                    },
                )
            }
        }
    }
}
