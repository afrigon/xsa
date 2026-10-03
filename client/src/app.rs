use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, bail, ensure};
use glam::{DVec2, DVec3, Vec3};
use tokio::sync::mpsc::UnboundedReceiver;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};
use xsa_commands::command::{CameraAction, CameraMode as CommandCameraMode, CameraTarget, ClientCommand};
use xsa_commands::router::{CommandExecution, CommandExecutor, CommandRouter};
use xsa_commands::target::Target;
use xsa_core::packs::{Id, PackStack};
use xsa_core::simulation::{Simulation, SimulationState};
use xsa_proto::messages::{ClientMessage, PackReference, ServerEvent};
use xsa_proto::session::ServerSession;

use crate::camera::debug::DebugCamera;
use crate::camera::orbit::{OrbitCamera, OrbitTarget};
use crate::camera::{Camera, CameraMode};
use crate::content::{self, MaterialDefinition};
use crate::input::Input;
use crate::lighting::{self, StarLight};
use crate::renderer::{
    ColorSpace, HapkeParameters, Material, MaterialHandle, ObjectHandle, Renderer, SceneObject, Shader,
};

const APP_ID: &str = "xsa";
const BASE_PACK: &str = "base";
const HORIZONTAL_FIELD_OF_VIEW_DEGREES: f32 = 100.0;
const DEBUG_CAMERA_SPEED: f64 = 2_000_000.0;
const EXPOSURE_STEP_STOPS: f32 = 1.0 / 3.0;
const BLOOM_STRENGTH_STEP_STOPS: f32 = 0.5;
const METERS_PER_KILOMETER: f64 = 1_000.0;

struct ShaderShortcut {
    key: KeyCode,
    shader: Option<Shader>,
}

const SHADER_SHORTCUTS: [ShaderShortcut; 5] = [
    ShaderShortcut {
        key: KeyCode::Digit1,
        shader: None,
    },
    ShaderShortcut {
        key: KeyCode::Digit2,
        shader: Some(Shader::Normals),
    },
    ShaderShortcut {
        key: KeyCode::Digit3,
        shader: Some(Shader::Depth),
    },
    ShaderShortcut {
        key: KeyCode::Digit4,
        shader: Some(Shader::Triangles),
    },
    ShaderShortcut {
        key: KeyCode::Digit5,
        shader: Some(Shader::Lighting),
    },
];

struct World {
    simulation: Simulation,
    state: SimulationState,
    body_objects: Vec<ObjectHandle>,
    light_body: Option<usize>,
}

pub struct App {
    renderer: Option<Renderer>,
    window: Option<Window>,
    error: Option<anyhow::Error>,
    session: ServerSession,
    router: CommandRouter,
    invocations: UnboundedReceiver<CommandExecution>,
    exit_requested: bool,
    packs_directory: PathBuf,
    world: Option<World>,
    skybox: Option<MaterialHandle>,
    input: Input,
    camera: Camera,
    camera_mode: CameraMode,
    orbit_camera: Option<OrbitCamera>,
    debug_camera: DebugCamera,
    mouse_captured: bool,
    last_frame: Option<Instant>,
}

impl App {
    pub fn new(
        session: ServerSession,
        invocations: UnboundedReceiver<CommandExecution>,
        packs_directory: PathBuf,
    ) -> Self {
        Self {
            renderer: None,
            window: None,
            error: None,
            session,
            router: CommandRouter::default(),
            invocations,
            exit_requested: false,
            packs_directory,
            world: None,
            skybox: None,
            input: Input::default(),
            camera: Camera::new(HORIZONTAL_FIELD_OF_VIEW_DEGREES.to_radians()),
            camera_mode: CameraMode::Orbit,
            orbit_camera: None,
            debug_camera: DebugCamera::new(DEBUG_CAMERA_SPEED),
            mouse_captured: false,
            last_frame: None,
        }
    }

    pub fn into_result(self) -> anyhow::Result<()> {
        self.error.map_or(Ok(()), Err)
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let window = event_loop.create_window(window_attributes())?;
        let base = PackStack::load(&self.packs_directory, &[BASE_PACK.to_string()])?;
        let shaders = content::load_shaders(&base)?;
        self.renderer = Some(Renderer::new(&window, &shaders)?);
        window.request_redraw();
        self.window = Some(window);
        Ok(())
    }

    fn redraw(&mut self) -> anyhow::Result<()> {
        let now = Instant::now();
        let delta_seconds = self.last_frame.map_or(0.0, |last| (now - last).as_secs_f64());
        self.last_frame = Some(now);

        self.update(delta_seconds)?;
        self.draw()?;
        self.input.end_frame();

        if let Some(window) = &self.window {
            window.request_redraw();
        }
        Ok(())
    }

    fn update(&mut self, delta_seconds: f64) -> anyhow::Result<()> {
        self.poll_connection()?;
        self.poll_invocations();
        self.poll_input();
        self.update_simulation();
        self.update_camera(delta_seconds);
        Ok(())
    }

    fn poll_connection(&mut self) -> anyhow::Result<()> {
        while let Some(event) = self.session.poll()? {
            self.router.handle_event(&event, &self.session);
            match event {
                ServerEvent::JoinAccepted { state } => self
                    .join(&state.simulation, &state.packs, state.time)
                    .with_context(|| format!("joining simulation {}", state.simulation))?,
                ServerEvent::JoinDenied { reason } => bail!("the server refused to let us join: {reason}"),
                ServerEvent::PlayerJoined { .. }
                | ServerEvent::PlayerLeft { .. }
                | ServerEvent::TimeChanged { .. }
                | ServerEvent::Tick { .. }
                | ServerEvent::Reply { .. } => {}
            }
        }
        Ok(())
    }

    fn poll_invocations(&mut self) {
        if self.world.is_none() {
            return;
        }
        let mut router = std::mem::take(&mut self.router);
        while let Ok(invocation) = self.invocations.try_recv() {
            router.execute(invocation, self);
        }
        self.router = router;
    }

    fn join(&mut self, simulation: &str, packs: &[PackReference], time: f64) -> anyhow::Result<()> {
        let renderer = self.renderer.as_mut().context("the renderer is not ready")?;
        let pack_ids: Vec<String> = packs.iter().map(|pack| pack.id.clone()).collect();
        let stack = PackStack::load(&self.packs_directory, &pack_ids)?;
        for (expected, installed) in packs.iter().zip(stack.manifests()) {
            ensure!(
                installed.version.to_string() == expected.version,
                "the server uses {} {}, but {} is installed",
                expected.id,
                expected.version,
                installed.version
            );
        }

        let simulation_id = Id::parse(simulation, BASE_PACK)?;
        let simulation = Simulation::load(&stack, &simulation_id)?;
        let star_lights: Vec<Option<StarLight>> = simulation
            .bodies()
            .iter()
            .map(|body| body.star.map(|star| lighting::star_light(&star, body.radius)))
            .collect();
        let materials = simulation
            .bodies()
            .iter()
            .zip(&star_lights)
            .map(|(body, star_light)| {
                let definition = content::load_body_material(&stack, &body.id)?;
                create_material(renderer, &body.id, definition, star_light.as_ref())
                    .with_context(|| format!("loading the material of {}", body.id))
            })
            .collect::<anyhow::Result<Vec<Material>>>()?;
        let mut state = SimulationState::default();
        simulation.state_at(time, &mut state);

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
        let light_body = star_lights.iter().position(Option::is_some);
        if let Some(index) = light_body
            && let Some(star_light) = &star_lights[index]
        {
            scene.sun_intensity = star_light.color * star_light.luminous_intensity as f32;
        }
        let target = simulation
            .spawn()
            .or_else(|| star_lights.iter().position(Option::is_none))
            .unwrap_or(0);

        self.skybox = match content::load_skybox(&stack, &simulation_id)? {
            Some(skybox) => match renderer.load_cube_map("skybox", &skybox.texture) {
                Ok(cube_map) => Some(renderer.scene_mut().add_material(Material::Skybox {
                    cube_map,
                    orientation: skybox.orientation,
                    luminance: skybox.luminance,
                })),
                Err(err) => {
                    eprintln!("skipping the skybox: {err:#}");
                    None
                }
            },
            None => None,
        };
        renderer.scene_mut().skybox = self.skybox;

        if let Some(body) = simulation.bodies().get(target) {
            self.orbit_camera = Some(OrbitCamera::new(target, body.radius));
        }
        self.world = Some(World {
            simulation,
            state,
            body_objects,
            light_body,
        });
        Ok(())
    }

    fn poll_input(&mut self) {
        if self.input.was_pressed(KeyCode::Escape) {
            self.set_mouse_captured(false);
        }
        if self.input.was_pressed(KeyCode::F1) {
            self.toggle_camera_mode();
            println!("camera: {}", self.camera_mode.name());
        }
        if self.camera_mode == CameraMode::Orbit && self.input.was_pressed(KeyCode::Tab) {
            let backwards = self.input.is_held(KeyCode::ShiftLeft) || self.input.is_held(KeyCode::ShiftRight);
            self.cycle_target(if backwards { -1 } else { 1 });
        }
        self.poll_mouse_capture();
        self.poll_render_settings();
    }

    fn poll_mouse_capture(&mut self) {
        match self.camera_mode {
            CameraMode::Debug if self.input.was_button_pressed(MouseButton::Left) => self.set_mouse_captured(true),
            CameraMode::Orbit if self.input.was_button_pressed(MouseButton::Right) => self.set_mouse_captured(true),
            CameraMode::Orbit if self.input.was_button_released(MouseButton::Right) => self.set_mouse_captured(false),
            _ => {}
        }
    }

    fn poll_render_settings(&mut self) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        for shortcut in &SHADER_SHORTCUTS {
            if self.input.was_pressed(shortcut.key) {
                renderer.set_shader_override(shortcut.shader);
                println!("view: {}", shortcut.shader.map_or("shaded", Shader::path));
            }
        }
        if self.input.was_pressed(KeyCode::Backquote) {
            match renderer.toggle_wireframe() {
                Some(enabled) => println!("wireframe: {}", on_off(enabled)),
                None => println!("wireframe: unsupported by this device"),
            }
        }
        if self.input.was_pressed(KeyCode::F2) {
            let scene = renderer.scene_mut();
            scene.skybox = if scene.skybox.is_some() { None } else { self.skybox };
            println!("skybox: {}", on_off(scene.skybox.is_some()));
        }
        if self.input.was_pressed(KeyCode::Minus) {
            renderer.adjust_exposure(-EXPOSURE_STEP_STOPS);
            print_exposure(renderer);
        }
        if self.input.was_pressed(KeyCode::Equal) {
            renderer.adjust_exposure(EXPOSURE_STEP_STOPS);
            print_exposure(renderer);
        }
        if self.input.was_pressed(KeyCode::F4) {
            renderer.toggle_auto_exposure();
            print_exposure(renderer);
        }
        if self.input.was_pressed(KeyCode::F5) {
            let enabled = renderer.toggle_bloom();
            println!("bloom: {}", on_off(enabled));
        }
        if self.input.was_pressed(KeyCode::BracketLeft) {
            println!(
                "bloom strength: {:.4}",
                renderer.adjust_bloom_strength(-BLOOM_STRENGTH_STEP_STOPS)
            );
        }
        if self.input.was_pressed(KeyCode::BracketRight) {
            println!(
                "bloom strength: {:.4}",
                renderer.adjust_bloom_strength(BLOOM_STRENGTH_STEP_STOPS)
            );
        }
        if self.input.was_pressed(KeyCode::F7) {
            println!("shading: {}", renderer.cycle_shading_model().name());
        }
        if self.input.was_pressed(KeyCode::F3) {
            println!("tonemapper: {}", renderer.cycle_tonemapper().name());
        }
    }

    fn update_simulation(&mut self) {
        if let Some(world) = &mut self.world
            && let Some(state) = self.session.state()
        {
            world.simulation.state_at(state.time(), &mut world.state);
        }
    }

    fn update_camera(&mut self, delta_seconds: f64) {
        let Some(world) = &self.world else {
            return;
        };
        match self.camera_mode {
            CameraMode::Orbit => {
                if let Some(orbit_camera) = &mut self.orbit_camera {
                    let target = orbit_camera.target();
                    let target = OrbitTarget {
                        position: world.state.bodies[target].position,
                        radius: world.simulation.bodies()[target].radius,
                    };
                    orbit_camera.update(&mut self.camera, &self.input, target, delta_seconds);
                }
            }
            CameraMode::Debug => self.debug_camera.update(&mut self.camera, &self.input, delta_seconds),
        }
        let nearest_surface_distance = world
            .simulation
            .bodies()
            .iter()
            .zip(&world.state.bodies)
            .map(|(body, state)| state.position.distance(self.camera.position) - body.radius)
            .fold(f64::INFINITY, f64::min);
        self.camera.fit_near_plane(nearest_surface_distance);
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.sync_scene();
        match &mut self.renderer {
            Some(renderer) => renderer.draw(&self.camera),
            None => Ok(()),
        }
    }

    fn sync_scene(&mut self) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        let Some(world) = &self.world else {
            return;
        };
        let scene = renderer.scene_mut();
        for ((body, state), &handle) in world
            .simulation
            .bodies()
            .iter()
            .zip(&world.state.bodies)
            .zip(&world.body_objects)
        {
            let object = scene.object_mut(handle);
            object.position = state.position;
            object.orientation = state.orientation;
            object.scale = body.radius;
        }
        if let Some(light_body) = world.light_body {
            scene.sun_position = world.state.bodies[light_body].position;
        }
    }

    fn set_mouse_captured(&mut self, captured: bool) {
        let Some(window) = &self.window else {
            return;
        };
        if captured {
            let grabbed = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            if grabbed.is_err() {
                return;
            }
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
        }
        window.set_cursor_visible(!captured);
        self.mouse_captured = captured;
    }

    fn toggle_camera_mode(&mut self) {
        self.set_camera_mode(match self.camera_mode {
            CameraMode::Orbit => CameraMode::Debug,
            CameraMode::Debug => CameraMode::Orbit,
        });
    }

    fn set_camera_mode(&mut self, mode: CameraMode) {
        if self.camera_mode == mode {
            return;
        }
        self.set_mouse_captured(false);
        if mode == CameraMode::Debug
            && let Some(orbit_camera) = &self.orbit_camera
        {
            self.debug_camera.look_along(orbit_camera.yaw(), orbit_camera.pitch());
        }
        self.camera_mode = mode;
    }

    fn cycle_target(&mut self, step: isize) {
        let target = if step < 0 { Target::Previous } else { Target::Next };
        match self.target_camera(CameraTarget {
            target,
            distance: None,
            pitch: None,
            yaw: None,
        }) {
            Ok(description) => println!("{description}"),
            Err(err) => eprintln!("{err:#}"),
        }
    }

    fn target_camera(&mut self, request: CameraTarget) -> anyhow::Result<String> {
        let world = self.world.as_ref().context("not joined to a simulation yet")?;
        let orbit_camera = self.orbit_camera.as_mut().context("the simulation has no bodies")?;
        let bodies = world.simulation.bodies();
        let target = match &request.target {
            Target::Next => (orbit_camera.target() + 1) % bodies.len(),
            Target::Previous => (orbit_camera.target() + bodies.len() - 1) % bodies.len(),
            Target::Body { id } => find_body(&world.simulation, id)?,
        };
        let body = &bodies[target];
        if target != orbit_camera.target() {
            orbit_camera.set_target(target, world.state.bodies[target].position, body.radius);
        }
        if let Some(distance) = request.distance {
            orbit_camera.set_distance(distance.meters);
        }
        if let Some(pitch) = request.pitch {
            orbit_camera.set_pitch(pitch.to_radians());
        }
        if let Some(yaw) = request.yaw {
            orbit_camera.set_yaw(yaw.to_radians());
        }
        let description = format!(
            "target: {}, distance {:.0} km, pitch {:.1}°, yaw {:.1}°",
            body.id.path,
            orbit_camera.distance() / METERS_PER_KILOMETER,
            orbit_camera.pitch().to_degrees(),
            orbit_camera.yaw().to_degrees()
        );
        self.set_camera_mode(CameraMode::Orbit);
        Ok(description)
    }

    fn look_at(&mut self, target: &Target) -> anyhow::Result<String> {
        ensure!(
            self.camera_mode == CameraMode::Debug,
            "look-at turns the debug camera; switch to it with `camera mode debug`"
        );
        let Target::Body { id } = target else {
            bail!("look-at needs a body id");
        };
        let world = self.world.as_ref().context("not joined to a simulation yet")?;
        let body = find_body(&world.simulation, id)?;
        let direction = (world.state.bodies[body].position - self.camera.position).normalize_or_zero();
        ensure!(direction != DVec3::ZERO, "the camera is at the center of {id}");
        self.debug_camera
            .look_along((-direction.x).atan2(direction.y), direction.z.asin());
        Ok(format!("looking at {id}"))
    }
}

impl CommandExecutor for App {
    fn session(&mut self) -> &mut ServerSession {
        &mut self.session
    }

    fn exit(&mut self) {
        self.exit_requested = true;
    }

    fn run_client(&mut self, command: ClientCommand) -> anyhow::Result<String> {
        match command {
            ClientCommand::Camera(CameraAction::Mode { mode }) => {
                let description = match mode {
                    CommandCameraMode::Target => {
                        self.set_camera_mode(CameraMode::Orbit);
                        "camera: target"
                    }
                    CommandCameraMode::Debug => {
                        self.set_camera_mode(CameraMode::Debug);
                        "camera: debug"
                    }
                };
                Ok(description.to_string())
            }
            ClientCommand::Camera(CameraAction::Target(request)) => self.target_camera(request),
            ClientCommand::Camera(CameraAction::LookAt { target }) => self.look_at(&target),
        }
    }
}

fn find_body(simulation: &Simulation, id: &str) -> anyhow::Result<usize> {
    simulation
        .bodies()
        .iter()
        .position(|body| body.id.path == id || body.id.to_string() == id)
        .with_context(|| format!("no body {id} in {}", simulation.id()))
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        if let Err(err) = self.initialize(event_loop) {
            self.error = Some(err);
            event_loop.exit();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                let _ = self.session.send(ClientMessage::Leave);
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::Focused(false) => self.input.clear(),
            WindowEvent::KeyboardInput { event, .. } => self.input.handle_key(&event),
            WindowEvent::MouseInput { state, button, .. } => self.input.handle_mouse_button(button, state),
            WindowEvent::MouseWheel { delta, .. } => self.input.handle_scroll(delta),
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.redraw() {
                    self.error = Some(err);
                    event_loop.exit();
                } else if self.exit_requested {
                    let _ = self.session.send(ClientMessage::Leave);
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event
            && self.mouse_captured
        {
            self.input.handle_mouse_motion(DVec2::from(delta));
        }
    }
}

fn window_attributes() -> WindowAttributes {
    let attributes = Window::default_attributes().with_title(APP_ID);
    #[cfg(all(unix, not(target_vendor = "apple")))]
    let attributes = winit::platform::wayland::WindowAttributesExtWayland::with_name(attributes, APP_ID, APP_ID);
    attributes
}

fn on_off(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

fn print_exposure(renderer: &Renderer) {
    if renderer.auto_exposure_enabled() {
        println!(
            "exposure: auto, EV100 {:.2}, compensation {:+.2}",
            renderer.exposure_ev100(),
            renderer.exposure_compensation()
        );
    } else {
        println!("exposure: manual, EV100 {:.2}", renderer.exposure_ev100());
    }
}

fn create_material(
    renderer: &mut Renderer,
    body: &Id,
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
