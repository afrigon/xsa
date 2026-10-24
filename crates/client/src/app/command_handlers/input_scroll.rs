use xsa_commands::command::InputScrollCommand;
use xsa_input::ScrollDelta;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InputScrollCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.inject_scroll(self.amount)?;
        let amount = match self.amount {
            ScrollDelta::Notches { vertical } => format!("{vertical} notches"),
            ScrollDelta::Pixels { vertical } => format!("{vertical}px"),
        };

        Ok(format!("input: scrolled {amount}"))
    }
}
