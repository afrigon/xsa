use xsa_commands::command::InputUpCommand;
use xsa_input::InputEventKind;

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InputUpCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        let released = self.chord.pressed_buttons().into_iter().rev();
        app.inject(released.map(|button| InputEventKind::Released { button }))?;

        Ok(format!("input: released {}", self.chord.name()))
    }
}
