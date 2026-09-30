use std::time::Instant;

use glam::{DQuat, DVec3};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use crate::camera::Camera;
use crate::camera::debug::DebugCamera;
use crate::input::Input;
use crate::renderer::{PLANET_RADIUS, Renderer};

const START_DISTANCE: f64 = 20_000_000.0;
const START_SPEED: f64 = 1_000_000.0;
const NEAR_PLANE_ALTITUDE_FRACTION: f64 = 0.1;
const MINIMUM_NEAR_PLANE: f64 = 0.1;

pub struct App {
    renderer: Option<Renderer>,
    window: Option<Window>,
    error: Option<anyhow::Error>,
    input: Input,
    camera: Camera,
    debug_camera: DebugCamera,
    mouse_captured: bool,
    last_frame: Option<Instant>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            renderer: None,
            window: None,
            error: None,
            input: Input::default(),
            camera: Camera {
                position: DVec3::new(0.0, -START_DISTANCE, 0.0),
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
        self.renderer = Some(Renderer::new(&window)?);
        window.request_redraw();
        self.window = Some(window);
        self.set_mouse_captured(true);
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

    fn redraw(&mut self) -> anyhow::Result<()> {
        let now = Instant::now();
        let delta_seconds = self.last_frame.map_or(0.0, |last| (now - last).as_secs_f64());
        self.last_frame = Some(now);

        self.debug_camera.update(&mut self.camera, &self.input, delta_seconds);
        let altitude = self.camera.position.length() - PLANET_RADIUS;
        self.camera.near = (altitude * NEAR_PLANE_ALTITUDE_FRACTION).max(MINIMUM_NEAR_PLANE) as f32;
        self.input.end_frame();

        if let Some(renderer) = &mut self.renderer {
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
                if event.physical_key == PhysicalKey::Code(KeyCode::Escape) && event.state.is_pressed() {
                    self.set_mouse_captured(false);
                }
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
