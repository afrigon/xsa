use std::path::Path;
use std::time::Instant;

use glam::{DQuat, DVec2, DVec3, Vec3};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};

use crate::camera::debug::DebugCamera;
use crate::camera::orbit::{OrbitCamera, OrbitTarget};
use crate::camera::{Camera, CameraMode};
use crate::input::Input;
use crate::renderer::{Material, MaterialHandle, ObjectHandle, Renderer, SceneObject, Shader};
use xsa_core::frames;
use xsa_proto::connection::Connection;
use xsa_proto::messages::{BodyDefinition, ClientMessage, ServerEvent};

const APP_ID: &str = "xsa";
const HORIZONTAL_FIELD_OF_VIEW_DEGREES: f32 = 100.0;
const DEBUG_CAMERA_SPEED: f64 = 2_000_000.0;
const SOL_COLOR: Vec3 = Vec3::new(1.0, 0.95, 0.85);
const EARTH_COLOR: Vec3 = Vec3::new(0.1, 0.25, 0.6);
const LUNA_COLOR: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const DEFAULT_COLOR: Vec3 = Vec3::new(0.6, 0.6, 0.6);
const SKYBOX_PATH: &str = "assets/deep-star-maps/skybox.dds";

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

struct ReplicatedBody {
    id: String,
    position: DVec3,
    orientation: DQuat,
    radius: f64,
}

pub struct App {
    renderer: Option<Renderer>,
    window: Option<Window>,
    error: Option<anyhow::Error>,
    connection: Connection,
    bodies: Vec<ReplicatedBody>,
    body_objects: Vec<ObjectHandle>,
    skybox: Option<MaterialHandle>,
    sun_body: Option<usize>,
    input: Input,
    camera: Camera,
    camera_mode: CameraMode,
    orbit_camera: Option<OrbitCamera>,
    debug_camera: DebugCamera,
    mouse_captured: bool,
    last_frame: Option<Instant>,
}

impl App {
    pub fn new(connection: Connection) -> Self {
        Self {
            renderer: None,
            window: None,
            error: None,
            connection,
            bodies: Vec::new(),
            body_objects: Vec::new(),
            skybox: None,
            sun_body: None,
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
        let mut renderer = Renderer::new(&window)?;
        self.skybox = load_skybox(&mut renderer);
        renderer.scene_mut().skybox = self.skybox;
        self.renderer = Some(renderer);
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
        self.poll_input();
        self.update_camera(delta_seconds);
        Ok(())
    }

    fn poll_connection(&mut self) -> anyhow::Result<()> {
        while let Some(event) = self.connection.poll()? {
            match event {
                ServerEvent::Welcome { bodies } => self.welcome(bodies),
                ServerEvent::Tick { .. } => {}
            }
        }
        Ok(())
    }

    fn welcome(&mut self, definitions: Vec<BodyDefinition>) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        self.bodies = definitions
            .into_iter()
            .map(|definition| ReplicatedBody {
                id: definition.id,
                position: DVec3::from_array(definition.position),
                orientation: DQuat::from_array(definition.orientation),
                radius: definition.radius,
            })
            .collect();
        self.body_objects = self
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
        self.sun_body = self.bodies.iter().position(|body| body.id == "sol");
        let target = self.bodies.iter().position(|body| body.id == "earth").unwrap_or(0);
        self.orbit_camera = self
            .bodies
            .get(target)
            .map(|body| OrbitCamera::new(target, body.radius));
    }

    fn poll_input(&mut self) {
        if self.input.was_pressed(KeyCode::Escape) {
            self.set_mouse_captured(false);
        }
        if self.input.was_pressed(KeyCode::F1) {
            self.toggle_camera_mode();
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
            }
        }
        if self.input.was_pressed(KeyCode::Backquote) {
            renderer.toggle_wireframe();
        }
        if self.input.was_pressed(KeyCode::F2) {
            let scene = renderer.scene_mut();
            scene.skybox = if scene.skybox.is_some() { None } else { self.skybox };
        }
    }

    fn update_camera(&mut self, delta_seconds: f64) {
        match self.camera_mode {
            CameraMode::Orbit => {
                if let Some(orbit_camera) = &mut self.orbit_camera {
                    let target = &self.bodies[orbit_camera.target()];
                    let target = OrbitTarget {
                        position: target.position,
                        radius: target.radius,
                    };
                    orbit_camera.update(&mut self.camera, &self.input, target, delta_seconds);
                }
            }
            CameraMode::Debug => self.debug_camera.update(&mut self.camera, &self.input, delta_seconds),
        }
        let nearest_surface_distance = self
            .bodies
            .iter()
            .map(|body| body.position.distance(self.camera.position) - body.radius)
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
        let scene = renderer.scene_mut();
        for (body, &handle) in self.bodies.iter().zip(&self.body_objects) {
            let object = scene.object_mut(handle);
            object.position = body.position;
            object.orientation = body.orientation;
            object.scale = body.radius;
        }
        if let Some(sun_body) = self.sun_body {
            scene.sun_position = self.bodies[sun_body].position;
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
        self.set_mouse_captured(false);
        self.camera_mode = match self.camera_mode {
            CameraMode::Orbit => {
                if let Some(orbit_camera) = &self.orbit_camera {
                    self.debug_camera.look_along(orbit_camera.yaw(), orbit_camera.pitch());
                }
                CameraMode::Debug
            }
            CameraMode::Debug => CameraMode::Orbit,
        };
    }

    fn cycle_target(&mut self, step: isize) {
        let Some(orbit_camera) = &mut self.orbit_camera else {
            return;
        };
        let body_count = self.bodies.len() as isize;
        let target = (orbit_camera.target() as isize + step).rem_euclid(body_count) as usize;
        let body = &self.bodies[target];
        orbit_camera.set_target(target, body.position, body.radius);
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
            WindowEvent::CloseRequested => {
                let _ = self.connection.send(ClientMessage::Leave);
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

fn appearance(body: &ReplicatedBody) -> Material {
    match body.id.as_str() {
        "sol" => Material::Emissive { color: SOL_COLOR },
        "earth" => Material::Lit {
            base_color: EARTH_COLOR,
        },
        "luna" => Material::Lit { base_color: LUNA_COLOR },
        _ => Material::Lit {
            base_color: DEFAULT_COLOR,
        },
    }
}

fn load_skybox(renderer: &mut Renderer) -> Option<MaterialHandle> {
    match renderer.load_cube_map("skybox", Path::new(SKYBOX_PATH)) {
        Ok(cube_map) => Some(renderer.scene_mut().add_material(Material::Skybox {
            cube_map,
            orientation: frames::equatorial_from_ecliptic().as_mat3(),
        })),
        Err(err) => {
            eprintln!("skipping the skybox: {err:#}");
            None
        }
    }
}
