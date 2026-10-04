use xui::{Presentation, View, ViewController};

use super::PauseView;
use crate::ui::Actions;

// The pause menu, over the game. While it is in the stack, time stops when the server is the integrated one; a
// remote server keeps running for everyone else.
pub struct PauseViewController {
    pub actions: Actions,
    pub pauses_time: bool,
}

impl ViewController for PauseViewController {
    fn root(&self) -> impl View {
        PauseView
    }

    fn presentation(&self) -> Presentation {
        Presentation::Overlay
    }

    fn did_load(&mut self) {
        if self.pauses_time {
            self.actions.pause_time();
        }
    }

    fn did_unload(&mut self) {
        if self.pauses_time {
            self.actions.resume_time();
        }
    }
}
