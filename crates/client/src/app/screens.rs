use tokio::sync::oneshot;
use xsa_commands::command::{Overlay, Screen};

use super::App;
use crate::ui::GameViewController;

impl App {
    pub(super) fn push_screen(&mut self, screen: Screen) {
        self.navigation.push(self.screens.build(screen));
    }

    pub(super) fn pop_screen(&mut self) -> bool {
        self.navigation.pop()
    }

    pub(super) fn set_screen(&mut self, screen: Screen) {
        self.navigation.set_root(self.screens.build(screen));
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

    // The player plays only while the game is the visible top screen and nothing is changing screens: then the
    // camera gets input and may capture the mouse.
    pub(super) fn is_game_active(&self) -> bool {
        self.navigation.top().is::<GameViewController>() && !self.navigation.is_transitioning()
    }

    // Runs what the interface asked for this frame, through the router like any other command.
    pub(super) fn run_interface_actions(&mut self) {
        let mut router = std::mem::take(&mut self.router);

        for command in self.actions.take() {
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
