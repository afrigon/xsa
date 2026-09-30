use std::path::Path;
use std::time::Instant;

use glam::{DQuat, DVec3, Mat3, Vec3};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

use crate::camera::debug::DebugCamera;
use crate::camera::orbit::{OrbitCamera, OrbitTarget};
use crate::camera::{Camera, CameraMode};
use crate::input::Input;
use crate::renderer::{CubeMapSource, Material, MaterialHandle, ObjectHandle, Renderer, SceneObject, Shader};
use crate::simulation::{Body, Simulation};

const APP_ID: &str = "xsa";
const START_SPEED: f64 = 2_000_000.0;
const NEAR_PLANE_ALTITUDE_FRACTION: f64 = 0.1;
const MINIMUM_NEAR_PLANE: f64 = 0.1;
const SUN_COLOR: Vec3 = Vec3::new(1.0, 0.95, 0.85);
const EARTH_COLOR: Vec3 = Vec3::new(0.1, 0.25, 0.6);
const MOON_COLOR: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const DEFAULT_COLOR: Vec3 = Vec3::new(0.6, 0.6, 0.6);
const EARTH_OBLIQUITY_DEGREES: f32 = 23.439_281;
const SOL_SKYBOX_FACES: [&str; 6] = [
    "assets/sol/GalaxyTex_PositiveX.dds",
    "assets/sol/GalaxyTex_NegativeX.dds",
    "assets/sol/GalaxyTex_PositiveY.dds",
    "assets/sol/GalaxyTex_NegativeY.dds",
    "assets/sol/GalaxyTex_PositiveZ.dds",
    "assets/sol/GalaxyTex_NegativeZ.dds",
];
const DEEP_STAR_MAPS_SKYBOX: &str = "assets/deep-star-maps/skybox.dds";

pub struct App {
    renderer: Option<Renderer>,
    window: Option<Window>,
    error: Option<anyhow::Error>,
    simulation: Simulation,
    body_objects: Vec<ObjectHandle>,
    skyboxes: Vec<MaterialHandle>,
    skybox_choice: usize,
    sun_body: Option<usize>,
    input: Input,
    camera: Camera,
    camera_mode: CameraMode,
    orbit_camera: OrbitCamera,
    debug_camera: DebugCamera,
    mouse_captured: bool,
    last_frame: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        let simulation = Simulation::solar_system();
        let earth_body = simulation.bodies.iter().position(|body| body.name == "Earth").unwrap_or(0);
        let orbit_camera = OrbitCamera::new(earth_body, simulation.bodies[earth_body].radius);
        let sun_body = simulation.bodies.iter().position(|body| body.name == "Sun");
        Self {
            renderer: None,
            window: None,
            error: None,
            simulation,
            body_objects: Vec::new(),
            skyboxes: Vec::new(),
            skybox_choice: 0,
            sun_body,
            input: Input::default(),
            camera: Camera {
                position: DVec3::ZERO,
                orientation: DQuat::IDENTITY,
                horizontal_fov: 100_f32.to_radians(),
                near: MINIMUM_NEAR_PLANE as f32,
            },
            camera_mode: CameraMode::Orbit,
            orbit_camera,
            debug_camera: DebugCamera::new(START_SPEED),
            mouse_captured: false,
            last_frame: None,
        }
    }
}

impl App {
    pub fn into_result(self) -> anyhow::Result<()> {
        self.error.map_or(Ok(()), Err)
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let window = event_loop.create_window(window_attributes())?;
        let mut renderer = Renderer::new(&window)?;
        self.body_objects = self
            .simulation
            .bodies
            .iter()
            .map(|body| {
                let scene = renderer.scene_mut();
                let material = scene.add_material(appearance(body));
                scene.add(SceneObject {
                    position: body.position,
                    orientation: body.orientation,
                    scale: body.radius,
                    material,
                })
            })
            .collect();
        self.skyboxes = load_skyboxes(&mut renderer);
        renderer.scene_mut().skybox = self.skyboxes.first().copied();
        self.renderer = Some(renderer);
        window.request_redraw();
        self.window = Some(window);
        Ok(())
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

    fn handle_hotkey(&mut self, event: &KeyEvent) {
        if !event.state.is_pressed() || event.repeat {
            return;
        }
        let PhysicalKey::Code(key) = event.physical_key else {
            return;
        };
        if key == KeyCode::Escape {
            self.set_mouse_captured(false);
            return;
        }
        match key {
            KeyCode::F1 => return self.toggle_camera_mode(),
            KeyCode::Tab if self.camera_mode == CameraMode::Orbit => {
                let shifted = self.input.is_held(KeyCode::ShiftLeft) || self.input.is_held(KeyCode::ShiftRight);
                return self.cycle_target(if shifted { -1 } else { 1 });
            }
            _ => {}
        }
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        match key {
            KeyCode::Digit1 => renderer.set_shader_override(None),
            KeyCode::Digit2 => renderer.set_shader_override(Some(Shader::Normals)),
            KeyCode::Digit3 => renderer.set_shader_override(Some(Shader::Depth)),
            KeyCode::Digit4 => renderer.set_shader_override(Some(Shader::Triangles)),
            KeyCode::Digit5 => renderer.set_shader_override(Some(Shader::Lighting)),
            KeyCode::Backquote => renderer.toggle_wireframe(),
            KeyCode::F2 => {
                self.skybox_choice = (self.skybox_choice + 1) % (self.skyboxes.len() + 1);
                renderer.scene_mut().skybox = self.skyboxes.get(self.skybox_choice).copied();
            }
            _ => {}
        }
    }

    fn handle_mouse_capture(&mut self, button: MouseButton, state: ElementState) {
        match (self.camera_mode, button) {
            (CameraMode::Debug, MouseButton::Left) if state.is_pressed() => self.set_mouse_captured(true),
            (CameraMode::Orbit, MouseButton::Right) => self.set_mouse_captured(state.is_pressed()),
            _ => {}
        }
    }

    fn toggle_camera_mode(&mut self) {
        self.set_mouse_captured(false);
        self.camera_mode = match self.camera_mode {
            CameraMode::Orbit => {
                self.debug_camera
                    .look_along(self.orbit_camera.yaw(), self.orbit_camera.pitch());
                CameraMode::Debug
            }
            CameraMode::Debug => CameraMode::Orbit,
        };
    }

    fn cycle_target(&mut self, step: isize) {
        let body_count = self.simulation.bodies.len() as isize;
        let target = (self.orbit_camera.target() as isize + step).rem_euclid(body_count) as usize;
        let body = &self.simulation.bodies[target];
        self.orbit_camera.set_target(target, body.position, body.radius);
    }

    fn redraw(&mut self) -> anyhow::Result<()> {
        let now = Instant::now();
        let delta_seconds = self.last_frame.map_or(0.0, |last| (now - last).as_secs_f64());
        self.last_frame = Some(now);

        match self.camera_mode {
            CameraMode::Orbit => {
                let target = &self.simulation.bodies[self.orbit_camera.target()];
                let target = OrbitTarget {
                    position: target.position,
                    radius: target.radius,
                };
                self.orbit_camera
                    .update(&mut self.camera, &self.input, target, delta_seconds);
            }
            CameraMode::Debug => self.debug_camera.update(&mut self.camera, &self.input, delta_seconds),
        }
        let nearest_altitude = self
            .simulation
            .bodies
            .iter()
            .map(|body| body.position.distance(self.camera.position) - body.radius)
            .fold(f64::INFINITY, f64::min);
        self.camera.near = (nearest_altitude * NEAR_PLANE_ALTITUDE_FRACTION).max(MINIMUM_NEAR_PLANE) as f32;
        self.input.end_frame();

        if let Some(renderer) = &mut self.renderer {
            let scene = renderer.scene_mut();
            for (body, &handle) in self.simulation.bodies.iter().zip(&self.body_objects) {
                let object = scene.object_mut(handle);
                object.position = body.position;
                object.orientation = body.orientation;
                object.scale = body.radius;
            }
            if let Some(sun_body) = self.sun_body {
                scene.sun_position = self.simulation.bodies[sun_body].position;
            }
            renderer.draw(&self.camera)?;
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        Ok(())
    }
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
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::Focused(false) => self.input.clear(),
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_hotkey(&event);
                self.input.handle_key(&event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input.handle_mouse_button(button, state);
                self.handle_mouse_capture(button, state);
            }
            WindowEvent::MouseWheel { delta, .. } => self.input.handle_scroll(delta),
            WindowEvent::RedrawRequested => {
                if let Err(err) = self.redraw() {
                    self.error = Some(err);
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
            self.input.handle_mouse_motion(delta);
        }
    }
}

fn window_attributes() -> WindowAttributes {
    let attributes = Window::default_attributes().with_title(APP_ID);
    #[cfg(all(unix, not(target_vendor = "apple")))]
    let attributes = winit::platform::wayland::WindowAttributesExtWayland::with_name(attributes, APP_ID, APP_ID);
    attributes
}

fn appearance(body: &Body) -> Material {
    match body.name.as_str() {
        "Sun" => Material::Emissive { color: SUN_COLOR },
        "Earth" => Material::Lit { base_color: EARTH_COLOR },
        "Moon" => Material::Lit { base_color: MOON_COLOR },
        _ => Material::Lit { base_color: DEFAULT_COLOR },
    }
}

fn load_skyboxes(renderer: &mut Renderer) -> Vec<MaterialHandle> {
    let sol = CubeMapSource::Faces(SOL_SKYBOX_FACES.map(Path::new));
    let sol_orientation = Mat3::from_cols(Vec3::X, Vec3::Z, Vec3::Y);
    let deep_star_maps = CubeMapSource::Single(Path::new(DEEP_STAR_MAPS_SKYBOX));
    let equatorial_from_ecliptic = Mat3::from_rotation_x(EARTH_OBLIQUITY_DEGREES.to_radians());

    [("Sol skybox", sol, sol_orientation), ("Deep Star Maps skybox", deep_star_maps, equatorial_from_ecliptic)]
        .into_iter()
        .filter_map(|(name, source, orientation)| match renderer.load_cube_map(name, source) {
            Ok(cube_map) => Some(renderer.scene_mut().add_material(Material::Skybox { cube_map, orientation })),
            Err(err) => {
                eprintln!("skipping the {name}: {err:#}");
                None
            }
        })
        .collect()
}
