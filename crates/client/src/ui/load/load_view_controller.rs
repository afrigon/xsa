use xui::{View, ViewController};

use super::LoadView;

pub struct LoadViewController;

impl ViewController for LoadViewController {
    fn root(&self) -> impl View {
        LoadView
    }
}
