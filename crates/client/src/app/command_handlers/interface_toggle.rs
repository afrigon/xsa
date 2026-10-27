use anyhow::ensure;
use xsa_commands::command::{InterfaceElement, InterfaceToggleCommand};

use crate::app::App;
use crate::app::immediate_command_handler::ImmediateCommandHandler;

impl ImmediateCommandHandler for InterfaceToggleCommand {
    fn run(self, app: &mut App) -> anyhow::Result<String> {
        match self.element {
            InterfaceElement::Overlay { overlay } => {
                let shown = !app.is_overlay_shown(overlay);
                app.set_overlay_shown(overlay, shown);
                let state = if shown { "showing" } else { "hiding" };

                Ok(format!("ui: {state} {overlay:?}"))
            }
            InterfaceElement::ViewController { view_controller } if app.is_on_top(view_controller) => {
                ensure!(
                    app.pop_view_controller(),
                    "the interface is already at its root view controller"
                );

                Ok(format!("ui: popped {view_controller:?}"))
            }
            InterfaceElement::ViewController { view_controller } => {
                app.check_push(view_controller)?;
                app.push_view_controller(view_controller);

                Ok(format!("ui: pushed {view_controller:?}"))
            }
        }
    }
}
