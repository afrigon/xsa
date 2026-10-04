use xui::{View, ViewController};

use super::MainMenuView;

pub struct MainMenuViewController;

impl ViewController for MainMenuViewController {
    fn root(&self) -> impl View {
        MainMenuView
    }
}
