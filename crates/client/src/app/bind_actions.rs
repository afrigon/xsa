use xsa_commands::command::{CameraTargetCommand, InterfacePushCommand, InterfaceToggleCommand, Overlay, Screen};
use xsa_commands::value::Target;

use super::App;
use super::client_command_handler::ClientCommandHandler;
use crate::config::BindAction;

impl App {
    pub(super) fn run_bind_action(&mut self, action: BindAction) {
        let progress = match action {
            BindAction::CameraTargetNext => App::target(Target::Next).start(self),
            BindAction::CameraTargetPrevious => App::target(Target::Previous).start(self),
            BindAction::InterfacePauseMenu if self.is_game_active() => InterfacePushCommand {
                screen: Screen::PauseMenu,
            }
            .start(self),
            BindAction::InterfacePauseMenu => return,
            BindAction::InterfaceDebugOverlay => InterfaceToggleCommand {
                overlay: Overlay::DebugOverlay,
            }
            .start(self),
        };

        self.follow(progress, None);
    }

    fn target(target: Target) -> CameraTargetCommand {
        CameraTargetCommand {
            target,
            category: None,
            distance: None,
            pitch: None,
            yaw: None,
        }
    }
}
