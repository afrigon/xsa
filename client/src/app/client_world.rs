use std::path::{Path, PathBuf};

use anyhow::{Context, ensure};
use glam::{DVec3, Vec3};
use xsa_commands::completion::CompletionCandidate;
use xsa_core::simulation::{Body, BodyId, BodyIndex, BodyState, Simulation, SimulationState};
use xsa_packs::{Id, MaterialDefinition, PackStack, SimulationDefinition, SkyboxDefinition};
use xsa_proto::event::WorldState;
use xsa_units::SimulationTime;

use crate::renderer::{ColorSpace, HapkeParameters, Material, MaterialHandle, ObjectHandle, Renderer, SceneObject};
use crate::star_light::StarLight;

const DEFAULT_NAMESPACE: &str = "base";

pub(super) struct ClientWorld {
    simulation_id: Id,
    simulation: Simulation,
    state: SimulationState,
    body_objects: Vec<ObjectHandle>,
    light_body: Option<BodyIndex>,
    skybox: Option<MaterialHandle>,
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
        let star_lights: Vec<Option<StarLight>> = simulation
            .bodies()
            .iter()
            .map(|body| body.star.map(|star| StarLight::of_star(&star, body.radius)))
            .collect();
        let materials = simulation
            .bodies()
            .iter()
            .zip(&star_lights)
            .map(|(body, star_light)| {
                let definition =
                    MaterialDefinition::load_for_body(&stack, &Id::parse(&body.id.value, DEFAULT_NAMESPACE)?)?;
                ClientWorld::create_material(renderer, &body.id, definition, star_light.as_ref())
                    .with_context(|| format!("loading the material of {}", body.id))
            })
            .collect::<anyhow::Result<Vec<Material>>>()?;
        let mut state = SimulationState::default();
        simulation.state_at(world.time, &mut state);

        let scene = renderer.scene_mut();
        let body_objects = simulation
            .bodies()
            .iter()
            .zip(&materials)
            .zip(&state.bodies)
            .map(|((body, material), body_state)| {
                let material = scene.add_material(*material);

                scene.add(SceneObject {
                    position: body_state.position,
                    orientation: body_state.orientation,
                    scale: body.radius,
                    material,
                })
            })
            .collect();
        let light_body = star_lights
            .iter()
            .position(Option::is_some)
            .map(|value| BodyIndex { value });

        if let Some(index) = light_body
            && let Some(star_light) = &star_lights[index.value]
        {
            scene.sun_intensity = star_light.color * star_light.luminous_intensity as f32;
        }

        let skybox = ClientWorld::load_skybox(&stack, &simulation_id, renderer)?;
        renderer.scene_mut().skybox = skybox;

        Ok(ClientWorld {
            simulation_id,
            simulation,
            state,
            body_objects,
            light_body,
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

    pub fn nearest_surface_distance(&self, position: DVec3) -> f64 {
        self.bodies()
            .iter()
            .zip(&self.state.bodies)
            .map(|(body, state)| state.position.distance(position) - body.radius)
            .fold(f64::INFINITY, f64::min)
    }

    pub fn body_candidates(&self) -> Vec<CompletionCandidate> {
        self.bodies()
            .iter()
            .map(|body| CompletionCandidate {
                value: body.id.to_string(),
                description: None,
            })
            .collect()
    }

    pub fn advance(&mut self, time: SimulationTime) {
        self.simulation.state_at(time, &mut self.state);
    }

    pub fn sync(&self, renderer: &mut Renderer) {
        let scene = renderer.scene_mut();

        for ((body, state), &handle) in self.bodies().iter().zip(&self.state.bodies).zip(&self.body_objects) {
            let object = scene.object_mut(handle);
            object.position = state.position;
            object.orientation = state.orientation;
            object.scale = body.radius;
        }

        if let Some(light_body) = self.light_body {
            scene.sun_position = self.state.body(light_body).position;
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
            Ok(cube_map) => Ok(Some(renderer.scene_mut().add_material(Material::Skybox {
                cube_map,
                orientation: skybox.orientation,
                luminance: skybox.luminance,
            }))),
            Err(err) => {
                tracing::warn!("skipping the skybox: {err:#}");
                Ok(None)
            }
        }
    }

    fn create_material(
        renderer: &mut Renderer,
        body: &BodyId,
        definition: MaterialDefinition,
        star_light: Option<&StarLight>,
    ) -> anyhow::Result<Material> {
        Ok(match definition {
            MaterialDefinition::Lit { base_color } => Material::Lit { base_color },
            MaterialDefinition::Emissive { color, luminance } => {
                let luminance = match star_light {
                    Some(star_light) => star_light.color * star_light.surface_luminance as f32,
                    None => Vec3::splat(luminance.context("an emissive material needs a `luminance` in cd/m²")?),
                };

                Material::Emissive {
                    luminance: color * luminance,
                }
            }
            MaterialDefinition::Planet {
                color,
                normal,
                emissive,
                emissive_luminance,
                hapke,
            } => {
                let mut load = |kind: &str, path: &PathBuf, color_space| {
                    renderer.load_texture(&format!("{body} {kind}"), path, color_space)
                };

                Material::Planet {
                    color: load("color", &color, ColorSpace::Srgb)?,
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
                                scatter: load("scatter", &hapke.scatter, ColorSpace::Linear)?,
                                surge: load("surge", &hapke.surge, ColorSpace::Linear)?,
                                porosity: hapke.porosity,
                                roughness: hapke.roughness_degrees.to_radians(),
                                blend: hapke.blend,
                                light_boost: hapke.light_boost,
                                gamma_boost: hapke.gamma_boost,
                            })
                        })
                        .transpose()?,
                }
            }
        })
    }
}
