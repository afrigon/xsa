use xui::{View, ViewController};

use super::ConfigView;

pub struct ConfigViewController;

impl ViewController for ConfigViewController {
    fn root(&self) -> impl View {
        ConfigView
    }
}
