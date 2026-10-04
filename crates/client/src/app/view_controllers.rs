use tokio::sync::oneshot;
use xsa_commands::command::{Overlay, ViewControllerId};

use super::App;
use crate::ui::GameViewController;

impl App {
    pub(super) fn push_view_controller(&mut self, id: ViewControllerId) {
        self.navigation.push(self.view_controllers.build(id));
    }

    pub(super) fn pop_view_controller(&mut self) -> bool {
        self.navigation.pop()
    }

    pub(super) fn set_view_controller(&mut self, id: ViewControllerId) {
        self.navigation.set_root(self.view_controllers.build(id));
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
