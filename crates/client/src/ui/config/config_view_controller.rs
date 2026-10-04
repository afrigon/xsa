use xui::{Controller, View, ViewController};

use super::ConfigView;

pub struct ConfigViewController;

impl ViewController for ConfigViewController {
    fn root(&self, _this: &Controller<Self>) -> impl View {
        ConfigView
    }
}
