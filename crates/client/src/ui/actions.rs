use std::cell::RefCell;
use std::rc::Rc;

use xsa_commands::command::{
    Command, ExitCommand, InterfaceAction, InterfaceCommand, InterfacePopCommand, InterfacePushCommand,
    InterfaceSetCommand, Screen, TimeAction, TimeCommand, TimePauseCommand, TimeResumeCommand,
};

// What the interface asks the game to do, like SwiftUI's environment actions: each method queues a typed command,
// which the game runs like any other command once per frame.
#[derive(Clone, Default)]
pub struct Actions {
    requests: Rc<RefCell<Vec<Command>>>,
}

impl Actions {
    pub fn push(&self, screen: Screen) {
        self.interface(InterfaceAction::Push(InterfacePushCommand { screen }));
    }

    pub fn pop(&self) {
        self.interface(InterfaceAction::Pop(InterfacePopCommand {}));
    }

    pub fn set(&self, screen: Screen) {
        self.interface(InterfaceAction::Set(InterfaceSetCommand { screen }));
    }

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
        std::mem::take(&mut self.requests.borrow_mut())
    }

    fn interface(&self, action: InterfaceAction) {
        self.send(Command::Ui(InterfaceCommand { action }));
    }

    fn time(&self, action: TimeAction) {
        self.send(Command::Time(TimeCommand { action: Some(action) }));
    }

    fn send(&self, command: Command) {
        self.requests.borrow_mut().push(command);
    }
}
