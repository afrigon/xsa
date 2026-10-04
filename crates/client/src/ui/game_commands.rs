use std::cell::RefCell;
use std::rc::Rc;

use xsa_commands::command::{Command, ExitCommand, TimeAction, TimeCommand, TimePauseCommand, TimeResumeCommand};

// What view controllers ask of the game beyond navigation: each method queues a typed command, which the game runs
// like any other command once per frame.
#[derive(Clone, Default)]
pub struct GameCommands {
    queue: Rc<RefCell<Vec<Command>>>,
}

impl GameCommands {
    pub fn pause_time(&self) {
        self.time(TimeAction::Pause(TimePauseCommand {}));
    }

    pub fn resume_time(&self) {
        self.time(TimeAction::Resume(TimeResumeCommand {}));
    }

    pub fn exit(&self) {
        self.send(Command::Exit(ExitCommand {}));
    }

    pub fn take(&self) -> Vec<Command> {
        std::mem::take(&mut self.queue.borrow_mut())
    }

    fn time(&self, action: TimeAction) {
        self.send(Command::Time(TimeCommand { action: Some(action) }));
    }

    fn send(&self, command: Command) {
        self.queue.borrow_mut().push(command);
    }
}
