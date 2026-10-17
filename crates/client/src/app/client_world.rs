use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, bail, ensure};
use glam::{DQuat, DVec3, Vec3};
use resource_loader::texture::{TextureFormat, TextureImage};
use xsa_commands::completion::CompletionCandidate;
use xsa_core::light::{Illumination, StarLight};
use xsa_core::simulation::{Body, BodyCategory, BodyId, BodyIndex, BodyState, Simulation, SimulationState};
use xsa_core::terrain::HeightMap;
use xsa_packs::{
    Id, LensDefinition, MaterialDefinition, PackStack, SimulationDefinition, SkyboxDefinition, TerrainDefinition,
};
use xsa_proto::event::WorldState;
use xsa_units::SimulationTime;

use super::body_step::BodyStep;
use crate::camera::Attitude;
use crate::renderer::{
    ColorSpace, EmissiveMaterial, HapkeParameters, LightSource, LitMaterial, MaterialHandle, ObjectHandle,
    PlanetMaterial, Renderer, SceneObject, SkyboxMaterial, Star, Terrain, TerrainHeight,
};
use crate::ui::SurfaceReading;

const DEFAULT_NAMESPACE: &str = "base";
const MAIN_LENS: &str = "main";
// Altitudes are read at the feet of a standing person, this far below the eyes.
const EYE_HEIGHT_METERS: f64 = 1.8;

pub(super) struct ClientWorld {
    simulation_id: Id,
    simulation: Simulation,
    state: SimulationState,
    illumination: Illumination,
    body_objects: Vec<ObjectHandle>,
    // One per body; None for a smooth body.
    height_maps: Vec<Option<Arc<HeightMap>>>,
    skybox: Option<MaterialHandle>,
}

struct LoadedTerrain {
    height: TerrainHeight,
    map: Arc<HeightMap>,
}

impl ClientWorld {
    pub fn load(packs_directory: &Path, world: &WorldState, renderer: &mut Renderer) -> anyhow::Result<ClientWorld> {
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

        let simulation_id = Id::parse(&world.simulation, DEFAULT_NAMESPACE)?;
        let simulation = stack.load_data::<SimulationDefinition>(&simulation_id)?.build(&stack)?;
        let illumination = Illumination::new(simulation.bodies());
        let materials = simulation
            .bodies()
            .iter()
            .enumerate()
            .map(|(value, body)| {
                let definition =
                    MaterialDefinition::load_for_body(&stack, &Id::parse(&body.id.value, DEFAULT_NAMESPACE)?)?;
                let star_light = illumination.star_light(BodyIndex { value });
                ClientWorld::create_material(renderer, &body.id, definition, star_light)
                    .with_context(|| format!("loading the material of {}", body.id))
            })
            .collect::<anyhow::Result<Vec<MaterialHandle>>>()?;
        let terrains = simulation
            .bodies()
            .iter()
            .map(|body| {
                let id = Id::parse(&body.id.value, DEFAULT_NAMESPACE)?;
                ClientWorld::load_terrain(&stack, &id, body.radius, renderer)
                    .with_context(|| format!("loading the terrain of {}", body.id))
            })
            .collect::<anyhow::Result<Vec<Option<LoadedTerrain>>>>()?;
        let mut heights = Vec::with_capacity(terrains.len());
        let mut height_maps = Vec::with_capacity(terrains.len());

        for terrain in terrains {
            match terrain {
                Some(terrain) => {
                    heights.push(Some(terrain.height));
                    height_maps.push(Some(terrain.map));
                }
                None => {
                    heights.push(None);
                    height_maps.push(None);
                }
            }
        }
        let mut state = SimulationState::default();
        simulation.state_at(world.time, &mut state);

        let scene = renderer.scene_mut();
        let body_objects: Vec<ObjectHandle> = simulation
            .bodies()
            .iter()
            .zip(&materials)
            .zip(heights)
            .zip(&state.bodies)
            .map(|(((body, &material), height), body_state)| {
                let highest_point = body.radius + height.as_ref().map_or(0.0, TerrainHeight::maximum);
                let object = scene.add(SceneObject {
                    position: body_state.position,
                    orientation: body_state.orientation,
                    scale: body.radius,
                    bounding_radius: highest_point,
                });
                scene.add_terrain(Terrain {
                    object,
                    material,
                    height,
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

        let skybox = ClientWorld::load_skybox(&stack, &simulation_id, renderer)?;
        renderer.scene_mut().skybox = skybox;
        ClientWorld::load_lens(&stack, renderer);
        renderer.finish_uploads()?;

        Ok(ClientWorld {
            simulation_id,
            simulation,
            state,
            illumination,
            body_objects,
            height_maps,
            skybox,
        })
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

    // The body whose radius the position is closest to.
    pub fn nearest_body(&self, position: DVec3) -> Option<BodyIndex> {
        self.bodies()
            .iter()
            .zip(&self.state.bodies)
            .map(|(body, state)| state.position.distance(position) - body.radius)
            .enumerate()
            .min_by(|first, second| first.1.total_cmp(&second.1))
            .map(|(value, _)| BodyIndex { value })
    }

    // The camera against `index`: latitude, longitude and altitudes at the feet, EYE_HEIGHT_METERS below the camera,
    // and its attitude against the local horizon.
    pub fn surface(&self, index: BodyIndex, position: DVec3, orientation: DQuat) -> SurfaceReading {
        let body = self.body(index);
        let state = self.body_state(index);
        let offset = position - state.position;
        let direction = (state.orientation.inverse() * offset).normalize();
        let above_sea = offset.length() - body.radius - EYE_HEIGHT_METERS;
        let up = offset.normalize();
        let latitude = direction.z.clamp(-1.0, 1.0).asin();
        let longitude = direction.y.atan2(direction.x);

        SurfaceReading {
            body: body.id.to_string(),
            latitude,
            longitude,
            above_ground: self.height_maps[index.value]
                .as_ref()
                .map(|map| above_sea - map.height_at(direction)),
            above_sea,
            attitude: Attitude::of(orientation, up, self.local_north(index, latitude, longitude)),
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
        self.height_maps[index.value]
            .as_ref()
            .map_or(0.0, |map| map.height_at(direction))
    }

    // Where the eyes are when the feet stand `altitude` above the ground at `direction`, in the body's frame.
    pub fn eye_position(&self, index: BodyIndex, direction: DVec3, altitude: f64) -> DVec3 {
        let state = self.body_state(index);
        let distance = self.body(index).radius + self.ground_height(index, direction) + altitude + EYE_HEIGHT_METERS;

        state.position + state.orientation * (direction * distance)
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
    pub fn step_body(
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
    }

    pub fn sync(&self, renderer: &mut Renderer) {
        let scene = renderer.scene_mut();

        for (state, &handle) in self.state.bodies.iter().zip(&self.body_objects) {
            let object = scene.object_mut(handle);
            object.position = state.position;
            object.orientation = state.orientation;
        }
    }

    fn load_lens(stack: &PackStack, renderer: &mut Renderer) {
        let loaded = Id::parse(MAIN_LENS, DEFAULT_NAMESPACE)
            .and_then(|profile| LensDefinition::load(stack, &profile))
            .and_then(|definition| renderer.load_lens(&definition));

        if let Err(err) = loaded {
            tracing::warn!("skipping lens effects: lens {MAIN_LENS}: {err:#}");
        }
    }

    fn load_skybox(
        stack: &PackStack,
        simulation_id: &Id,
        renderer: &mut Renderer,
    ) -> anyhow::Result<Option<MaterialHandle>> {
        let Some(skybox) = SkyboxDefinition::load(stack, simulation_id)? else {
            return Ok(None);
        };

        match renderer.load_cube_map("skybox", &skybox.texture) {
            Ok(cube_map) => Ok(Some(renderer.add_material(
                "skybox",
                SkyboxMaterial {
                    cube_map,
                    orientation: skybox.orientation,
                    luminance: skybox.luminance,
                },
            )?)),
            Err(err) => {
                tracing::warn!("skipping the skybox: {err:#}");
                Ok(None)
            }
        }
    }

    // A terrain without its height map is drawn smooth, so it needs no height.
    fn load_terrain(
        stack: &PackStack,
        body: &Id,
        radius: f64,
        renderer: &mut Renderer,
    ) -> anyhow::Result<Option<LoadedTerrain>> {
        let Some(definition) = TerrainDefinition::load_for_body(stack, body)? else {
            return Ok(None);
        };
        let Some(path) = definition.height_map else {
            return Ok(None);
        };

        let image = TextureImage::read(&path, ColorSpace::Linear)?;
        ensure!(
            image.format() == TextureFormat::R16Unorm,
            "{}: a height map must be R16_UNORM, not {:?}",
            path.display(),
            image.format()
        );

        let width = image.width() as usize;
        let height = image.height() as usize;
        let texels = image.layers()[0][..width * height * size_of::<u16>()]
            .as_chunks::<{ size_of::<u16>() }>()
            .0
            .iter()
            .map(|&bytes| u16::from_le_bytes(bytes))
            .collect();
        let map = Arc::new(HeightMap::new(
            width,
            height,
            texels,
            definition.minimum_height,
            definition.maximum_height,
        )?);
        let height = renderer.add_terrain_height(&format!("{body} terrain"), image, Arc::clone(&map), radius)?;

        Ok(Some(LoadedTerrain { height, map }))
    }

    fn create_material(
        renderer: &mut Renderer,
        body: &BodyId,
        definition: MaterialDefinition,
        star_light: Option<&StarLight>,
    ) -> anyhow::Result<MaterialHandle> {
        let name = format!("{body} material");

        match definition {
            MaterialDefinition::Lit { base_color } => renderer.add_material(&name, LitMaterial { base_color }),
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
            MaterialDefinition::Planet {
                base_color,
                color,
                normal,
                emissive,
                emissive_luminance,
                hapke,
            } => {
                let mut load = |kind: &str, path: &PathBuf, color_space| {
                    renderer.load_texture(&format!("{body} {kind}"), path, color_space)
                };

                let material = PlanetMaterial {
                    base_color,
                    color: color.map(|path| load("color", &path, ColorSpace::Srgb)).transpose()?,
                    normal: normal
                        .map(|path| load("normal", &path, ColorSpace::Linear))
                        .transpose()?,
                    emissive: emissive
                        .map(|path| load("emissive", &path, ColorSpace::Srgb))
                        .transpose()?,
                    emissive_luminance,
                    hapke: hapke
                        .map(|hapke| -> anyhow::Result<HapkeParameters> {
                            Ok(HapkeParameters {
                                medium: load("hapke medium", &hapke.medium, ColorSpace::Linear)?,
                                opposition: load("hapke opposition", &hapke.opposition, ColorSpace::Linear)?,
                            })
                        })
                        .transpose()?,
                };
                renderer.add_material(&name, material)
            }
        }
    }
}
