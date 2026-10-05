use xsa_commands::command::{
    Availability, CameraTargetCommand, InterfacePushCommand, InterfaceToggleCommand, Overlay, Routable,
    ViewControllerId,
};
use xsa_commands::value::Target;

use super::App;
use super::client_command_handler::ClientCommandHandler;
use crate::config::BindAction;

impl App {
    pub(super) fn run_bind_action(&mut self, action: BindAction) {
        match action {
            BindAction::CameraTargetNext => self.run_bound(App::target(Target::Next)),
            BindAction::CameraTargetPrevious => self.run_bound(App::target(Target::Previous)),
            BindAction::InterfacePauseMenu if self.is_game_active() => self.run_bound(InterfacePushCommand {
                view_controller: ViewControllerId::PauseMenu,
            }),
            BindAction::InterfacePauseMenu => {}
            BindAction::InterfaceDebugOverlay => self.run_bound(InterfaceToggleCommand {
                overlay: Overlay::DebugOverlay,
            }),
        }
    }

    // A bind runs its command as if typed: one available only in game does nothing while a menu is open.
    fn run_bound(&mut self, command: impl ClientCommandHandler + Routable) {
        if command.availability() == Availability::InGame && !self.is_game_active() {
            return;
        }

        let progress = command.start(self);
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
