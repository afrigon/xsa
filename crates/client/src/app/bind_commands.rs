use tokio::sync::oneshot;
use xsa_commands::router::CommandExecution;

use super::App;
use super::held_bind::HeldBind;
use crate::config::BindCommand;

impl App {
    // Binds run their commands as typed ones run: through the router, logging a failure.
    pub(super) fn run_binds(&mut self) {
        let pressed: Vec<_> = self
            .config
            .input
            .bind
            .pressed(&self.input)
            .into_iter()
            .cloned()
            .collect();

        for bind in pressed {
            self.run_bind_command(&bind.down);

            if let Some(up) = bind.up {
                self.held_binds.push(HeldBind { key: bind.key, up });
            }
        }

        let released: Vec<_> = self.input.released().copied().collect();
        let (ended, held) = std::mem::take(&mut self.held_binds)
            .into_iter()
            .partition(|held: &HeldBind| released.iter().any(|button| held.key.is_triggered_by(*button)));
        self.held_binds = held;

        for held in ended {
            self.run_bind_command(&held.up);
        }
    }

    fn run_bind_command(&mut self, command: &BindCommand) {
        let mut router = std::mem::take(&mut self.router);
        let (reply, receiver) = oneshot::channel();
        let execution = CommandExecution {
            words: command.words().to_vec(),
            reply,
            styled: false,
        };
        router.execute(execution, self);
        self.router = router;
        self.action_replies.push(receiver);
    }
}
