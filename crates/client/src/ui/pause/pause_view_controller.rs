use xsa_commands::command::ViewControllerId;
use xui::{Controller, Navigation, Presentation, View, ViewController};

use super::{PauseActions, PauseView};
use crate::ui::ViewControllerFactory;

// The pause menu, over the game. While it is in the stack, time stops when the server is the integrated one; a
// remote server keeps running for everyone else.
pub struct PauseViewController {
    pub navigation: Navigation,
    pub factory: ViewControllerFactory,
}

impl ViewController for PauseViewController {
    fn root(&self, this: &Controller<Self>) -> impl View {
        PauseView {
            actions: PauseActions {
                options: this.action(Self::show_options),
                main_menu: this.action(Self::return_to_main_menu),
            },
        }
    }

    fn presentation(&self) -> Presentation {
        Presentation::Overlay
    }

    fn did_load(&mut self) {
        if self.factory.pauses_time {
            self.factory.commands.pause_time();
        }
    }

    fn did_unload(&mut self) {
        if self.factory.pauses_time {
            self.factory.commands.resume_time();
        }
    }
}

impl PauseViewController {
    fn show_options(&mut self) {
        let config = self.factory.build(ViewControllerId::Config);
        self.navigation.push(config);
    }

    fn return_to_main_menu(&mut self) {
        let main_menu = self.factory.build(ViewControllerId::MainMenu);
        self.navigation.set_root(main_menu);
    }
}
