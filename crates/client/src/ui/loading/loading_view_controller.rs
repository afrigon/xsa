use xui::{Controller, View, ViewController};

use super::LoadingView;

// Shown from Play until the scene's files are loaded; the app then replaces it with the game.
pub struct LoadingViewController;

impl ViewController for LoadingViewController {
    fn root(&self, _this: &Controller<Self>) -> impl View {
        LoadingView
    }
}
