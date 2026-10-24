use glam::DVec2;
use xsa_commands::command::InputCursorSetCommand;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;
use crate::app::glide_motion::GlideMotion;
use crate::app::input_glide::InputGlide;

impl ClientCommandHandler for InputCursorSetCommand {
    fn start(self, app: &mut App) -> CommandProgress {
        let position = DVec2::new(self.x, self.y);
        let summary = format!("input: cursor at {}, {}", self.x, self.y);

        let Some(duration) = self.duration else {
            return CommandProgress::Done(app.inject_cursor_position(position).map(|()| summary));
        };

        match app.cursor_offset_to(position) {
            Ok(offset) => InputGlide::start(
                app,
                GlideMotion::Mouse { offset },
                duration,
                self.easing.easing(),
                summary,
            ),
            Err(err) => CommandProgress::Done(Err(err)),
        }
    }
}
