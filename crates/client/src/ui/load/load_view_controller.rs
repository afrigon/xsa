use xui::{Controller, View, ViewController};

use super::LoadView;

pub struct LoadViewController;

impl ViewController for LoadViewController {
    fn root(&self, _this: &Controller<Self>) -> impl View {
        LoadView
    }
}
