use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};
use xui::{Point, PointerButton, PointerEvent, PointerEventKind};

use super::App;

// The interface sees the pointer first: what it uses never reaches the camera or the binds. While the camera holds
// the mouse captured, the interface sees nothing.
impl App {
    pub(super) fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor = Point {
            x: position.x as f32,
            y: position.y as f32,
        };
        self.pointer_moved = true;
    }

    pub(super) fn handle_cursor_left(&mut self) {
        self.pointer_moved = false;
        self.offer_pointer(PointerEventKind::Left);
    }

    // Moves arrive faster than frames and only the latest position matters for hover, so the interface sees at
    // most one move per frame, offered before it is updated.
    pub(super) fn offer_pointer_move(&mut self) {
        if self.pointer_moved && !self.mouse_captured {
            self.offer_pointer(PointerEventKind::Moved);
        }

        self.pointer_moved = false;
    }

    pub(super) fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let used = match App::pointer_button(button) {
            Some(button) if !self.mouse_captured => self.offer_pointer(match state {
                ElementState::Pressed => PointerEventKind::Pressed { button },
                ElementState::Released => PointerEventKind::Released { button },
            }),
            _ => false,
        };

        if !used {
            self.input.handle_mouse_button(button, state);
        }
    }

    fn offer_pointer(&mut self, kind: PointerEventKind) -> bool {
        self.interface.handle_event(PointerEvent {
            kind,
            position: self.cursor,
        })
    }

    fn pointer_button(button: MouseButton) -> Option<PointerButton> {
        match button {
            MouseButton::Left => Some(PointerButton::Primary),
            MouseButton::Right => Some(PointerButton::Secondary),
            MouseButton::Middle => Some(PointerButton::Middle),
            MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => None,
        }
    }
}
