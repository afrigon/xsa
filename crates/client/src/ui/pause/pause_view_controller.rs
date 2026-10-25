use xsa_commands::command::ViewControllerId;
use xsa_proto::request::{RequestType, SetTimeRateOverrideRequest};
use xsa_units::TimeRate;
use xui::{Controller, Environment, Navigation, Presentation, View, ViewController};

use super::{PauseActions, PauseView};
use crate::ui::{SessionEnvironmentKey, ViewControllerFactory};

// The pause menu, over the game. While it is in the stack, a time override stops time wherever the session allows
// one; elsewhere the server keeps running for everyone else.
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

    fn did_load(&mut self, environment: &Environment) {
        if Self::stops_time(environment) {
            self.factory.commands.override_time(TimeRate::PAUSED);
        }
    }

    fn did_unload(&mut self, environment: &Environment) {
        if Self::stops_time(environment) {
            self.factory.commands.clear_time_override();
        }
    }
}

impl PauseViewController {
    fn stops_time(environment: &Environment) -> bool {
        environment
            .get::<SessionEnvironmentKey>()
            .is_some_and(|session| SetTimeRateOverrideRequest::SESSION_ENVIRONMENTS.contains(session))
    }

    fn show_options(&mut self) {
        let config = self.factory.build(ViewControllerId::Config);
        self.navigation.push(config);
    }

    fn return_to_main_menu(&mut self) {
        let main_menu = self.factory.build(ViewControllerId::MainMenu);
        self.navigation.set_root(main_menu);
    }
}
