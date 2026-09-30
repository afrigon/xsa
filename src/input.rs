use std::collections::HashSet;

use glam::DVec2;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};

const PIXELS_PER_SCROLL_STEP: f64 = 50.0;

#[derive(Default)]
pub struct Input {
    held_keys: HashSet<KeyCode>,
    mouse_delta: DVec2,
    scroll_steps: f64,
}

impl Input {
    pub fn handle_key(&mut self, event: &KeyEvent) {
        let PhysicalKey::Code(key) = event.physical_key else {
            return;
        };
        match event.state {
            ElementState::Pressed => self.held_keys.insert(key),
            ElementState::Released => self.held_keys.remove(&key),
        };
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        self.mouse_delta += DVec2::new(delta.0, delta.1);
    }

    pub fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        self.scroll_steps += match delta {
            MouseScrollDelta::LineDelta(_, lines) => f64::from(lines),
            MouseScrollDelta::PixelDelta(position) => position.y / PIXELS_PER_SCROLL_STEP,
        };
    }

    pub fn is_held(&self, key: KeyCode) -> bool {
        self.held_keys.contains(&key)
    }

    pub fn mouse_delta(&self) -> DVec2 {
        self.mouse_delta
    }

    pub fn scroll_steps(&self) -> f64 {
        self.scroll_steps
    }

    pub fn end_frame(&mut self) {
        self.mouse_delta = DVec2::ZERO;
        self.scroll_steps = 0.0;
    }

    pub fn clear(&mut self) {
        self.held_keys.clear();
        self.end_frame();
    }
}
