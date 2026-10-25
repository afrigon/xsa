use std::collections::HashSet;

use glam::DVec2;
use xsa_input::{ButtonPress, InputButton, Key, MouseButton, ScrollDelta};

const PIXELS_PER_SCROLL_STEP: f64 = 50.0;

#[derive(Default)]
pub struct Input {
    held: Vec<InputButton>,
    presses: Vec<ButtonPress>,
    released: HashSet<InputButton>,
    mouse_delta: DVec2,
    scroll_steps: f64,
}

impl Input {
    pub fn handle_mouse_motion(&mut self, delta: DVec2) {
        self.mouse_delta += delta;
    }

    pub fn handle_scroll(&mut self, delta: ScrollDelta) {
        self.scroll_steps += match delta {
            ScrollDelta::Notches { vertical } => vertical,
            ScrollDelta::Pixels { vertical } => vertical / PIXELS_PER_SCROLL_STEP,
        };
    }

    pub fn is_held(&self, key: Key) -> bool {
        self.held.contains(&InputButton::Key(key))
    }

    pub fn axis(&self, positive: Key, negative: Key) -> f64 {
        let value = |key| if self.is_held(key) { 1.0 } else { 0.0 };

        value(positive) - value(negative)
    }

    pub fn is_button_held(&self, button: MouseButton) -> bool {
        self.held.contains(&InputButton::Mouse(button))
    }

    pub fn held(&self) -> &[InputButton] {
        &self.held
    }

    pub fn presses(&self) -> &[ButtonPress] {
        &self.presses
    }

    pub fn was_button_pressed(&self, button: MouseButton) -> bool {
        self.presses
            .iter()
            .any(|press| press.button == InputButton::Mouse(button))
    }

    pub fn was_button_released(&self, button: MouseButton) -> bool {
        self.released.contains(&InputButton::Mouse(button))
    }

    pub fn mouse_delta(&self) -> DVec2 {
        self.mouse_delta
    }

    pub fn scroll_steps(&self) -> f64 {
        self.scroll_steps
    }

    pub fn end_frame(&mut self) {
        self.presses.clear();
        self.released.clear();
        self.mouse_delta = DVec2::ZERO;
        self.scroll_steps = 0.0;
    }

    pub fn clear(&mut self) {
        self.held.clear();
        self.end_frame();
    }

    pub fn press(&mut self, button: InputButton) {
        let held = self.held.iter().copied().filter(|held| *held != button).collect();
        self.presses.push(ButtonPress { button, held });

        if !self.held.contains(&button) {
            self.held.push(button);
        }
    }

    pub fn release(&mut self, button: InputButton) {
        self.held.retain(|held| *held != button);
        self.released.insert(button);
    }
}
