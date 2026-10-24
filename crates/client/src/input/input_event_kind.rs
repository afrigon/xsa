use glam::DVec2;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::keyboard::PhysicalKey;

use crate::input::InputButton;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputEventKind {
    Pressed { button: InputButton },
    Released { button: InputButton },
    CursorMoved { position: PhysicalPosition<f64> },
    CursorLeft,
    Moved { delta: DVec2 },
    Scrolled { delta: MouseScrollDelta },
}

impl InputEventKind {
    pub fn from_window_event(event: &WindowEvent) -> Option<InputEventKind> {
        match event {
            WindowEvent::KeyboardInput { event, .. } if !event.repeat => {
                let PhysicalKey::Code(key) = event.physical_key else {
                    return None;
                };

                Some(InputEventKind::button(InputButton::Key(key), event.state))
            }
            WindowEvent::MouseInput { state, button, .. } => {
                Some(InputEventKind::button(InputButton::Mouse(*button), *state))
            }
            WindowEvent::CursorMoved { position, .. } => Some(InputEventKind::CursorMoved { position: *position }),
            WindowEvent::CursorLeft { .. } => Some(InputEventKind::CursorLeft),
            WindowEvent::MouseWheel { delta, .. } => Some(InputEventKind::Scrolled { delta: *delta }),
            _ => None,
        }
    }

    // A release or the cursor leaving only ends what an earlier event began, so it is never held back.
    pub fn ends_input(&self) -> bool {
        matches!(self, InputEventKind::Released { .. } | InputEventKind::CursorLeft)
    }

    fn button(button: InputButton, state: ElementState) -> InputEventKind {
        match state {
            ElementState::Pressed => InputEventKind::Pressed { button },
            ElementState::Released => InputEventKind::Released { button },
        }
    }
}
