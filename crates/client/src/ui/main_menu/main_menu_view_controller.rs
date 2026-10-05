use xsa_commands::command::ViewControllerId;
use xui::{Controller, Navigation, View, ViewController};

use super::{MainMenuActions, MainMenuView};
use crate::ui::ViewControllerFactory;

pub struct MainMenuViewController {
    pub navigation: Navigation,
    pub factory: ViewControllerFactory,
}

impl ViewController for MainMenuViewController {
    fn root(&self, this: &Controller<Self>) -> impl View {
        MainMenuView {
            actions: MainMenuActions {
                continue_game: this.action(Self::play),
                load: this.action(Self::show_load),
                new_game: this.action(Self::play),
                options: this.action(Self::show_options),
                quit: this.action(Self::quit),
            },
        }
    }

    fn did_load(&mut self) {
        self.factory.presence.menu_loaded();

        if self.factory.pauses_time {
            self.factory.commands.hold_time();
        }
    }

    fn did_unload(&mut self) {
        self.factory.presence.menu_unloaded();

        if self.factory.pauses_time {
            self.factory.commands.release_time();
        }
    }
}

impl MainMenuViewController {
    fn play(&mut self) {
        let game = self.factory.build(ViewControllerId::Game);
        self.navigation.set_root(game);
    }

    fn show_load(&mut self) {
        let load = self.factory.build(ViewControllerId::Load);
        self.navigation.push(load);
    }

    fn show_options(&mut self) {
        let config = self.factory.build(ViewControllerId::Config);
        self.navigation.push(config);
    }

    fn quit(&mut self) {
        self.factory.commands.exit();
    }
}
