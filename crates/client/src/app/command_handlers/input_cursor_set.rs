use glam::DVec2;
use xsa_commands::command::InputCursorSetCommand;
use xsa_input::InputChord;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;
use crate::app::glide_motion::GlideMotion;
use crate::app::input_glide::InputGlide;

impl ClientCommandHandler for InputCursorSetCommand {
    fn start(self, app: &mut App) -> CommandProgress {
        let position = DVec2::new(self.x, self.y);
        let summary = format!("input: cursor at {}, {}", self.x, self.y);

        let offset = match app.cursor_offset_to(position) {
            Ok(offset) => offset,
            Err(err) => return CommandProgress::Done(Err(err)),
        };
        let held = self.hold.as_ref().map(InputChord::pressed_buttons).unwrap_or_default();
        let summary = InputGlide::summary(summary, self.hold.as_ref());

        InputGlide::start(
            app,
            GlideMotion::Mouse { offset },
            held,
            self.duration,
            self.easing.easing(),
            summary,
        )
    }
}
