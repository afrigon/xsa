use xsa_commands::command::ViewControllerId;
use xui::{AnyViewController, Navigation};

use super::{
    ConfigViewController, GameCommands, GameViewController, LoadViewController, MainMenuViewController, MenuPresence,
    PauseViewController,
};

// Builds the view controller the `ui` commands name, handing it the navigation stack and the game's commands.
#[derive(Clone)]
pub struct ViewControllerFactory {
    pub navigation: Navigation,
    pub commands: GameCommands,
    pub pauses_time: bool,
    pub presence: MenuPresence,
}

impl ViewControllerFactory {
    pub fn build(&self, id: ViewControllerId) -> Box<dyn AnyViewController> {
        match id {
            ViewControllerId::MainMenu => MainMenuViewController {
                navigation: self.navigation.clone(),
                factory: self.clone(),
            }
            .into(),
            ViewControllerId::Config => ConfigViewController.into(),
            ViewControllerId::Load => LoadViewController.into(),
            ViewControllerId::Game => GameViewController.into(),
            ViewControllerId::PauseMenu => PauseViewController {
                navigation: self.navigation.clone(),
                factory: self.clone(),
            }
            .into(),
        }
    }
}
