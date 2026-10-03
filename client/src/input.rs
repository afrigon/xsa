use std::collections::HashSet;

use glam::DVec2;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};

const PIXELS_PER_SCROLL_STEP: f64 = 50.0;

#[derive(Default)]
pub struct Input {
    held_keys: HashSet<KeyCode>,
    held_buttons: HashSet<MouseButton>,
    pressed_keys: HashSet<KeyCode>,
    pressed_buttons: HashSet<MouseButton>,
    released_buttons: HashSet<MouseButton>,
    mouse_delta: DVec2,
    scroll_steps: f64,
}

impl Input {
    pub fn handle_key(&mut self, event: &KeyEvent) {
        let PhysicalKey::Code(key) = event.physical_key else {
            return;
        };
        match event.state {
            ElementState::Pressed => {
                if !event.repeat {
                    self.pressed_keys.insert(key);
                }
                self.held_keys.insert(key);
            }
            ElementState::Released => {
                self.held_keys.remove(&key);
            }
        }
    }

    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.held_buttons.insert(button);
                self.pressed_buttons.insert(button);
            }
            ElementState::Released => {
                self.held_buttons.remove(&button);
                self.released_buttons.insert(button);
            }
        }
    }

    pub fn handle_mouse_motion(&mut self, delta: DVec2) {
        self.mouse_delta += delta;
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

    pub fn axis(&self, positive: KeyCode, negative: KeyCode) -> f64 {
        let value = |key| if self.is_held(key) { 1.0 } else { 0.0 };

        value(positive) - value(negative)
    }

    pub fn is_button_held(&self, button: MouseButton) -> bool {
        self.held_buttons.contains(&button)
    }

    pub fn was_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn was_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn was_button_released(&self, button: MouseButton) -> bool {
        self.released_buttons.contains(&button)
    }

    pub fn mouse_delta(&self) -> DVec2 {
        self.mouse_delta
    }

    pub fn scroll_steps(&self) -> f64 {
        self.scroll_steps
    }

    pub fn end_frame(&mut self) {
        self.pressed_keys.clear();
        self.pressed_buttons.clear();
        self.released_buttons.clear();
        self.mouse_delta = DVec2::ZERO;
        self.scroll_steps = 0.0;
    }

    pub fn clear(&mut self) {
        self.held_keys.clear();
        self.held_buttons.clear();
        self.end_frame();
    }
}
