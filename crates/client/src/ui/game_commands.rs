use std::cell::RefCell;
use std::rc::Rc;

use xsa_commands::command::{Command, ExitCommand, TimeAction, TimeCommand, TimeOverrideCommand};
use xsa_commands::value::TimeOverride;
use xsa_units::TimeRate;

// What view controllers and binds ask of the game beyond navigation: each method queues a typed command, which the game runs
// like any other command once per frame.
#[derive(Clone, Default)]
pub struct GameCommands {
    queue: Rc<RefCell<Vec<Command>>>,
}

impl GameCommands {
    pub fn override_time(&self, rate: TimeRate) {
        self.set_time_override(Some(rate));
    }

    pub fn clear_time_override(&self) {
        self.set_time_override(None);
    }

    pub fn exit(&self) {
        self.send(Command::Exit(ExitCommand {}));
    }

    pub fn take(&self) -> Vec<Command> {
        std::mem::take(&mut self.queue.borrow_mut())
    }

    fn set_time_override(&self, rate: Option<TimeRate>) {
        self.time(TimeAction::Override(TimeOverrideCommand {
            rate: TimeOverride { rate },
        }));
    }

    fn time(&self, action: TimeAction) {
        self.send(Command::Time(TimeCommand { action: Some(action) }));
    }

    fn send(&self, command: Command) {
        self.queue.borrow_mut().push(command);
    }
}
