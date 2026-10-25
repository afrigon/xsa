use glam::DVec2;
use xsa_commands::command::InputCursorMoveCommand;
use xsa_input::InputChord;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;
use crate::app::glide_motion::GlideMotion;
use crate::app::input_glide::InputGlide;

impl ClientCommandHandler for InputCursorMoveCommand {
    fn start(self, app: &mut App) -> CommandProgress {
        let offset = DVec2::new(self.dx, self.dy);
        let summary = format!("input: moved the mouse by {}, {}", self.dx, self.dy);

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
