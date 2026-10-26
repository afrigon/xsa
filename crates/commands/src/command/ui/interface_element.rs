use std::str::FromStr;

use usage::spec::ValueEnum;

use super::{Overlay, ViewControllerId};

// What `ui toggle` shows or hides: an overlay, or a view controller pushed over the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceElement {
    Overlay { overlay: Overlay },
    ViewController { view_controller: ViewControllerId },
}

impl FromStr for InterfaceElement {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if let Some(overlay) = Overlay::from_choice(text) {
            return Ok(InterfaceElement::Overlay { overlay });
        }

        if let Some(view_controller) = ViewControllerId::from_choice(text) {
            return Ok(InterfaceElement::ViewController { view_controller });
        }

        let names: Vec<&str> = Overlay::CHOICES
            .iter()
            .chain(ViewControllerId::CHOICES)
            .copied()
            .collect();

        Err(format!(
            "{text:?} is not an overlay or a view controller: {}",
            names.join(", ")
        ))
    }
}
