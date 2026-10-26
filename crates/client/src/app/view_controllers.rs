use resource_vulkan::LoadPriority;
use tokio::sync::oneshot;
use xsa_commands::command::{Overlay, ViewControllerId};

use super::App;
use anyhow::ensure;

use crate::ui::{
    ConfigViewController, GameViewController, LoadViewController, LoadingViewController, MainMenuViewController,
    PauseViewController,
};

impl App {
    // The pause menu opens over the game only.
    pub(super) fn check_push(&self, id: ViewControllerId) -> anyhow::Result<()> {
        ensure!(
            id != ViewControllerId::PauseMenu || self.is_game_active(),
            "the pause menu opens over the game only"
        );

        Ok(())
    }

    pub(super) fn is_on_top(&self, id: ViewControllerId) -> bool {
        let top = self.navigation.top();
        let on_top = match id {
            ViewControllerId::MainMenu => top.is::<MainMenuViewController>(),
            ViewControllerId::Config => top.is::<ConfigViewController>(),
            ViewControllerId::Load => top.is::<LoadViewController>(),
            ViewControllerId::Game => top.is::<GameViewController>(),
            ViewControllerId::PauseMenu => top.is::<PauseViewController>(),
        };

        on_top && !self.navigation.is_transitioning()
    }

    pub(super) fn push_view_controller(&mut self, id: ViewControllerId) {
        let environment = self.environment();
        self.navigation.push(self.view_controllers.build(id), &environment);
    }

    pub(super) fn pop_view_controller(&mut self) -> bool {
        let environment = self.environment();
        self.navigation.pop(&environment)
    }

    pub(super) fn set_view_controller(&mut self, id: ViewControllerId) {
        let environment = self.environment();
        self.navigation.set_root(self.view_controllers.build(id), &environment);
    }

    // The loading screen raises the scene's loads, and gives way to the game once every one settled.
    pub(super) fn advance_loading(&mut self) {
        if !self.navigation.top().is::<LoadingViewController>() || self.navigation.is_transitioning() {
            return;
        }

        let (Some(world), Some(resources)) = (&mut self.world, &self.resources) else {
            return;
        };

        if world.progress().is_loaded() {
            self.set_view_controller(ViewControllerId::Game);
        } else {
            world.set_priority(resources, LoadPriority::High);
        }
    }

    pub(super) fn is_overlay_shown(&self, overlay: Overlay) -> bool {
        self.overlays.contains(&overlay)
    }

    pub(super) fn set_overlay_shown(&mut self, overlay: Overlay, shown: bool) {
        if shown {
            self.overlays.insert(overlay);
        } else {
            self.overlays.remove(&overlay);
        }
    }

    // The player plays only while the game is the visible top view controller and no transition runs: then the
    // camera gets input and may capture the mouse.
    pub(super) fn is_game_active(&self) -> bool {
        self.navigation.top().is::<GameViewController>() && !self.navigation.is_transitioning()
    }

    // Runs what view controllers asked of the game this frame, through the router like any other command.
    pub(super) fn run_game_commands(&mut self) {
        let mut router = std::mem::take(&mut self.router);

        for command in self.commands.take() {
            let (reply, receiver) = oneshot::channel();
            router.dispatch(command, reply, self);
            self.action_replies.push(receiver);
        }

        self.router = router;
        self.action_replies.retain_mut(|receiver| match receiver.try_recv() {
            Ok(output) => {
                if !output.succeeded {
                    tracing::warn!("{}", output.text);
                }

                false
            }
            Err(oneshot::error::TryRecvError::Empty) => true,
            Err(oneshot::error::TryRecvError::Closed) => false,
        });
    }
}
