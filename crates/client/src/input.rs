mod button_press;
mod chord_button;
mod input_button;
mod input_chord;
mod input_event;
mod input_event_kind;
mod input_source;
mod modifier;

pub use button_press::ButtonPress;
pub use chord_button::ChordButton;
pub use input_button::InputButton;
pub use input_chord::InputChord;
pub use input_event::InputEvent;
pub use input_event_kind::InputEventKind;
pub use input_source::InputSource;
pub use modifier::Modifier;

use std::collections::HashSet;

use glam::DVec2;
use winit::event::{MouseButton, MouseScrollDelta};
use winit::keyboard::KeyCode;

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

    pub fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        self.scroll_steps += match delta {
            MouseScrollDelta::LineDelta(_, lines) => f64::from(lines),
            MouseScrollDelta::PixelDelta(position) => position.y / PIXELS_PER_SCROLL_STEP,
        };
    }

    pub fn is_held(&self, key: KeyCode) -> bool {
        self.held.contains(&InputButton::Key(key))
    }

    pub fn axis(&self, positive: KeyCode, negative: KeyCode) -> f64 {
        let value = |key| if self.is_held(key) { 1.0 } else { 0.0 };

        value(positive) - value(negative)
    }

    pub fn is_button_held(&self, button: MouseButton) -> bool {
        self.held.contains(&InputButton::Mouse(button))
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
