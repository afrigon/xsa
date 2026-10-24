use xsa_input::{InputButton, InputEvent, InputEventKind};

use super::App;

impl App {
    pub(super) fn handle_input(&mut self, event: InputEvent) {
        if !self.config.input.accepts(&event) {
            return;
        }

        match event.kind {
            InputEventKind::Pressed {
                button: InputButton::Mouse(button),
            } => self.handle_mouse_pressed(button),
            InputEventKind::Released {
                button: InputButton::Mouse(button),
            } => self.handle_mouse_released(button),
            InputEventKind::Pressed { button } => self.input.press(button),
            InputEventKind::Released { button } => self.input.release(button),
            InputEventKind::CursorMoved { position } => self.handle_cursor_moved(position),
            InputEventKind::CursorLeft => self.handle_cursor_left(),
            InputEventKind::Moved { delta } => self.input.handle_mouse_motion(delta),
            InputEventKind::Scrolled { delta } => self.input.handle_scroll(delta),
        }
    }
}
