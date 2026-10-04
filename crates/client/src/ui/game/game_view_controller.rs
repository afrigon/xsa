use xui::{View, ViewController};

use super::GameView;

pub struct GameViewController;

impl ViewController for GameViewController {
    fn root(&self) -> impl View {
        GameView
    }
}
