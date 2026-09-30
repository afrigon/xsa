use std::time::Instant;

use glam::{DQuat, DVec3, Vec3};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use crate::camera::Camera;
use crate::camera::debug::DebugCamera;
use crate::input::Input;
use crate::renderer::{Material, ObjectHandle, Renderer, SceneObject};
use crate::simulation::{Body, Simulation};

const START_DISTANCE: f64 = 20_000_000.0;
const START_SPEED: f64 = 2_000_000.0;
const NEAR_PLANE_ALTITUDE_FRACTION: f64 = 0.1;
const MINIMUM_NEAR_PLANE: f64 = 0.1;
const SUN_COLOR: Vec3 = Vec3::new(1.0, 0.95, 0.85);
const EARTH_COLOR: Vec3 = Vec3::new(0.1, 0.25, 0.6);
const MOON_COLOR: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const DEFAULT_COLOR: Vec3 = Vec3::new(0.6, 0.6, 0.6);

pub struct App {
    renderer: Option<Renderer>,
    window: Option<Window>,
    error: Option<anyhow::Error>,
    simulation: Simulation,
    body_objects: Vec<ObjectHandle>,
    sun_body: Option<usize>,
    input: Input,
    camera: Camera,
    debug_camera: DebugCamera,
    mouse_captured: bool,
    last_frame: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        let simulation = Simulation::solar_system();
        let earth_position = simulation
            .bodies
            .iter()
            .find(|body| body.name == "Earth")
            .map_or(DVec3::ZERO, |earth| earth.position);
        let sun_body = simulation.bodies.iter().position(|body| body.name == "Sun");
        Self {
            renderer: None,
            window: None,
            error: None,
            simulation,
            body_objects: Vec::new(),
            sun_body,
            input: Input::default(),
            camera: Camera {
                position: earth_position + DVec3::new(0.0, -START_DISTANCE, 0.0),
                orientation: DQuat::IDENTITY,
                horizontal_fov: 100_f32.to_radians(),
                near: MINIMUM_NEAR_PLANE as f32,
            },
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
        let window = event_loop.create_window(Window::default_attributes().with_title("xsa"))?;
        let mut renderer = Renderer::new(&window)?;
        self.body_objects = self
            .simulation
            .bodies
            .iter()
            .map(|body| {
                let (material, color) = appearance(body);
                renderer.scene_mut().add(SceneObject {
                    position: body.position,
                    orientation: body.orientation,
                    scale: body.radius,
                    color,
                    material,
                })
            })
            .collect();
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
        let Some(renderer) = &mut self.renderer else {
            return;
        };
        match key {
            KeyCode::Digit1 => renderer.set_material_override(None),
            KeyCode::Digit2 => renderer.set_material_override(Some(Material::Normals)),
            KeyCode::Digit3 => renderer.set_material_override(Some(Material::Depth)),
            KeyCode::Digit4 => renderer.set_material_override(Some(Material::Triangles)),
            KeyCode::Digit5 => renderer.set_material_override(Some(Material::Lighting)),
            KeyCode::Backquote => renderer.toggle_wireframe(),
            _ => {}
        }
    }

    fn redraw(&mut self) -> anyhow::Result<()> {
        let now = Instant::now();
        let delta_seconds = self.last_frame.map_or(0.0, |last| (now - last).as_secs_f64());
        self.last_frame = Some(now);

        self.debug_camera.update(&mut self.camera, &self.input, delta_seconds);
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
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } if !self.mouse_captured => self.set_mouse_captured(true),
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

fn appearance(body: &Body) -> (Material, Vec3) {
    match body.name.as_str() {
        "Sun" => (Material::Emissive, SUN_COLOR),
        "Earth" => (Material::Lit, EARTH_COLOR),
        "Moon" => (Material::Lit, MOON_COLOR),
        _ => (Material::Lit, DEFAULT_COLOR),
    }
}
