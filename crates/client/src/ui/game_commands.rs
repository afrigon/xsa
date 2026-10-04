use std::cell::{Cell, RefCell};
use std::rc::Rc;

use xsa_commands::command::{Command, ExitCommand, TimeAction, TimeCommand, TimePauseCommand, TimeResumeCommand};

// What view controllers ask of the game beyond navigation: each method queues a typed command, which the game runs
// like any other command once per frame.
#[derive(Clone, Default)]
pub struct GameCommands {
    queue: Rc<RefCell<Vec<Command>>>,
    time_holds: Rc<Cell<usize>>,
}

impl GameCommands {
    // Time stays paused while any view controller holds it, so one that loads before another unloads (the main menu
    // replacing the pause menu) never lets it run in between.
    pub fn hold_time(&self) {
        let holds = self.time_holds.get();
        self.time_holds.set(holds + 1);

        if holds == 0 {
            self.time(TimeAction::Pause(TimePauseCommand {}));
        }
    }

    pub fn release_time(&self) {
        let holds = self.time_holds.get() - 1;
        self.time_holds.set(holds);

        if holds == 0 {
            self.time(TimeAction::Resume(TimeResumeCommand {}));
        }
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
