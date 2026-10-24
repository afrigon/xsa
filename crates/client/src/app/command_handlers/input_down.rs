use xsa_commands::command::InputDownCommand;
use xsa_input::InputEventKind;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InputDownCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let pressed = self.chord.pressed_buttons().into_iter();
        app.inject(pressed.map(|button| InputEventKind::Pressed { button }))?;

        Ok(format!("input: holding {}", self.chord.name()))
    }
}
