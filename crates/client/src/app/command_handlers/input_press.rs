use animation::Easing;
use xsa_commands::command::InputPressCommand;

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;
use crate::app::glide_motion::GlideMotion;
use crate::app::input_glide::InputGlide;

impl ClientCommandHandler for InputPressCommand {
    fn start(self, app: &mut App) -> CommandProgress {
        let held = self.chord.pressed_buttons();
        let summary = format!("input: pressed {}", self.chord.name());

        InputGlide::start(app, GlideMotion::Hold, held, self.duration, Easing::Linear, summary)
    }
}
