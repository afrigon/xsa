use xsa_commands::command::Screen;
use xui::AnyViewController;

use super::{
    Actions, ConfigViewController, GameViewController, LoadViewController, MainMenuViewController, PauseViewController,
};

// Builds the controller for each screen the `ui` commands name.
pub struct ScreenControllers {
    pub actions: Actions,
    pub pauses_time: bool,
}

impl ScreenControllers {
    pub fn build(&self, screen: Screen) -> Box<dyn AnyViewController> {
        match screen {
            Screen::MainMenu => MainMenuViewController.into(),
            Screen::Config => ConfigViewController.into(),
            Screen::Load => LoadViewController.into(),
            Screen::Game => GameViewController.into(),
            Screen::PauseMenu => PauseViewController {
                actions: self.actions.clone(),
                pauses_time: self.pauses_time,
            }
            .into(),
        }
    }
}
