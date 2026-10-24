use xsa_commands::command::{
    CameraTargetCommand, InterfacePopCommand, InterfacePushCommand, InterfaceToggleCommand, Overlay, ViewControllerId,
};

use super::App;
use super::body_step::BodyStep;
use super::client_command_handler::ClientCommandHandler;
use crate::config::BindAction;

impl App {
    pub(super) fn run_bind_action(&mut self, action: BindAction) {
        let progress = match action {
            BindAction::CameraTargetNext => App::target(BodyStep::Next).start(self),
            BindAction::CameraTargetPrevious => App::target(BodyStep::Previous).start(self),
            BindAction::InterfacePauseMenu if self.is_game_active() => InterfacePushCommand {
                view_controller: ViewControllerId::PauseMenu,
            }
            .start(self),
            BindAction::InterfacePauseMenu if self.is_pause_menu_active() => InterfacePopCommand {}.start(self),
            BindAction::InterfacePauseMenu => return,
            BindAction::InterfaceDebugOverlay => InterfaceToggleCommand {
                overlay: Overlay::DebugOverlay,
            }
            .start(self),
            BindAction::TimeFaster => return self.commands.faster_time(),
            BindAction::TimeSlower => return self.commands.slower_time(),
            BindAction::TimeRealTime => return self.commands.resume_time(),
        };

        self.follow(progress, None);
    }

    fn target(step: BodyStep) -> CameraTargetCommand {
        CameraTargetCommand {
            body: None,
            next: step == BodyStep::Next,
            previous: step == BodyStep::Previous,
            parent: false,
            category: None,
            altitude: None,
            pitch: None,
            yaw: None,
        }
    }
}
