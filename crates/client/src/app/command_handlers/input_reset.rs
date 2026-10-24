use xsa_commands::command::InputResetCommand;
use xsa_input::InputEventKind;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InputResetCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        app.input_resets += 1;
        let held = app.input.held().to_vec();
        app.inject(held.into_iter().map(|button| InputEventKind::Released { button }))?;

        Ok("input: released every button".to_string())
    }
}
