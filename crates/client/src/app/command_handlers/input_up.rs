use xsa_commands::command::InputUpCommand;
use xsa_input::InputEventKind;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InputUpCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let buttons: Vec<_> = self.chord.buttons().collect();
        app.inject(buttons.into_iter().rev().map(|button| InputEventKind::Released {
            button: button.button(),
        }))?;

        Ok(format!("input: released {}", self.chord.name()))
    }
}
