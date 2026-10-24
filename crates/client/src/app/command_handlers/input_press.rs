use xsa_commands::command::InputPressCommand;
use xsa_input::InputEventKind;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InputPressCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let buttons: Vec<_> = self.chord.buttons().map(|button| button.button()).collect();
        let presses = buttons.iter().map(|button| InputEventKind::Pressed { button: *button });
        let releases = buttons
            .iter()
            .rev()
            .map(|button| InputEventKind::Released { button: *button });
        app.inject(presses.chain(releases))?;

        Ok(format!("input: pressed {}", self.chord.name()))
    }
}
