use xsa_commands::command::InputScrollCommand;
use xsa_input::{InputChord, ScrollDelta};

use crate::app::App;
use crate::app::client_command_handler::ClientCommandHandler;
use crate::app::command_progress::CommandProgress;
use crate::app::glide_motion::GlideMotion;
use crate::app::input_glide::InputGlide;

impl ClientCommandHandler for InputScrollCommand {
    fn start(self, app: &mut App) -> CommandProgress {
        let amount = match self.amount {
            ScrollDelta::Notches { vertical } => format!("{vertical} notches"),
            ScrollDelta::Pixels { vertical } => format!("{vertical}px"),
        };
        let summary = format!("input: scrolled {amount}");

        let held = self.hold.as_ref().map(InputChord::pressed_buttons).unwrap_or_default();
        let summary = InputGlide::summary(summary, self.hold.as_ref());

        InputGlide::start(
            app,
            GlideMotion::Scroll { amount: self.amount },
            held,
            self.duration,
            self.easing.easing(),
            summary,
        )
    }
}
