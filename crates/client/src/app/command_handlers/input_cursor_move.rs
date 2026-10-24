use glam::DVec2;
use xsa_commands::command::InputCursorMoveCommand;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;
use crate::app::glide_motion::GlideMotion;
use crate::app::input_glide::InputGlide;

impl ClientCommandHandler for InputCursorMoveCommand {
    fn start(self, app: &mut App) -> CommandProgress {
        let offset = DVec2::new(self.dx, self.dy);
        let summary = format!("input: moved the mouse by {}, {}", self.dx, self.dy);

        match self.duration {
            Some(duration) => InputGlide::start(
                app,
                GlideMotion::Mouse { offset },
                duration,
                self.easing.easing(),
                summary,
            ),
            None => CommandProgress::Done(app.inject_mouse_offset(offset).map(|()| summary)),
        }
    }
}
