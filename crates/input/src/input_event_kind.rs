use glam::DVec2;

use crate::{InputButton, ScrollDelta};

// Positions are physical pixels from the window's top left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputEventKind {
    Pressed { button: InputButton },
    Released { button: InputButton },
    CursorMoved { position: DVec2 },
    CursorLeft,
    Moved { delta: DVec2 },
    Scrolled { delta: ScrollDelta },
}

impl InputEventKind {
    // A release or the cursor leaving only ends what an earlier event began, so it is never held back.
    pub fn ends_input(&self) -> bool {
        matches!(self, InputEventKind::Released { .. } | InputEventKind::CursorLeft)
    }

    #[cfg(feature = "winit")]
    pub fn from_window_event(event: &winit::event::WindowEvent) -> Option<InputEventKind> {
        use winit::event::WindowEvent;
        use winit::keyboard::PhysicalKey;

        use crate::Key;

        match event {
            WindowEvent::KeyboardInput { event, .. } if !event.repeat => {
                let PhysicalKey::Code(key) = event.physical_key else {
                    return None;
                };

                Some(InputEventKind::button(
                    InputButton::Key(Key::from_winit(key)?),
                    event.state,
                ))
            }
            WindowEvent::MouseInput { state, button, .. } => {
                Some(InputEventKind::button(InputButton::Mouse((*button).into()), *state))
            }
            WindowEvent::CursorMoved { position, .. } => Some(InputEventKind::CursorMoved {
                position: DVec2::new(position.x, position.y),
            }),
            WindowEvent::CursorLeft { .. } => Some(InputEventKind::CursorLeft),
            WindowEvent::MouseWheel { delta, .. } => Some(InputEventKind::Scrolled { delta: (*delta).into() }),
            _ => None,
        }
    }

    #[cfg(feature = "winit")]
    fn button(button: InputButton, state: winit::event::ElementState) -> InputEventKind {
        match state {
            winit::event::ElementState::Pressed => InputEventKind::Pressed { button },
            winit::event::ElementState::Released => InputEventKind::Released { button },
        }
    }
}
