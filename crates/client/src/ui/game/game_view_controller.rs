use xui::{Controller, View, ViewController};

use super::GameView;

pub struct GameViewController;

impl ViewController for GameViewController {
    fn root(&self, _this: &Controller<Self>) -> impl View {
        GameView
    }
}
