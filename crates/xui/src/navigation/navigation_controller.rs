use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use super::{
    ActiveTransition, AnyViewController, ControllerMessage, DefaultNavigationDelegate, DismissKey, Navigation,
    NavigationDelegate, NavigationEntry, NavigationLayer, NavigationOperation, NavigationRequest, Presentation,
    TransitionAppearance, TransitionRole,
};
use crate::{Action, Alignment, ForEach, View, Visibility, ZStack};

// A stack of view controllers, like UIKit's `UINavigationController`. The app owns it and draws it with `view`;
// it changes through `push`, `pop` and `set_root`, or through the `Navigation` handles its view controllers hold.
// Every frame, `advance` runs the actions views triggered, applies the requested changes and moves transitions
// along. Changing the stack while a transition runs finishes that transition first.
pub struct NavigationController {
    entries: Vec<NavigationEntry>,
    next_id: u64,
    delegate: Box<dyn NavigationDelegate>,
    transition: Option<ActiveTransition>,
    requests: Rc<RefCell<Vec<NavigationRequest>>>,
    messages: Rc<RefCell<Vec<ControllerMessage>>>,
}

impl Default for NavigationController {
    fn default() -> NavigationController {
        NavigationController::new()
    }
}

impl NavigationController {
    // An empty stack: give it a root with `set_root` before drawing it. View controllers built for it take its
    // `navigation` handle.
    pub fn new() -> NavigationController {
        NavigationController {
            entries: Vec::new(),
            next_id: 0,
            delegate: Box::new(DefaultNavigationDelegate),
            transition: None,
            requests: Rc::default(),
            messages: Rc::default(),
        }
    }

    pub fn navigation(&self) -> Navigation {
        Navigation::new(self.requests.clone())
    }

    pub fn set_delegate(&mut self, delegate: impl NavigationDelegate) {
        self.delegate = Box::new(delegate);
    }

    pub fn push(&mut self, controller: impl Into<Box<dyn AnyViewController>>) {
        self.finish_transition();
        let visible_before = self.visible_ids(&[]);
        let outgoing = self.top_id();
        let incoming = self.insert(controller.into());
        self.begin(
            NavigationOperation::Push,
            outgoing,
            incoming,
            visible_before,
            Vec::new(),
        );
    }

    // Returns whether there was a view controller to pop: the root stays.
    pub fn pop(&mut self) -> bool {
        self.finish_transition();

        if self.entries.len() < 2 {
            return false;
        }

        let visible_before = self.visible_ids(&[]);
        let outgoing = self.top_id();
        let incoming = self.entries[self.entries.len() - 2].id;
        self.begin(
            NavigationOperation::Pop,
            outgoing,
            incoming,
            visible_before,
            vec![outgoing],
        );

        true
    }

    // Replaces the whole stack with `controller`.
    pub fn set_root(&mut self, controller: impl Into<Box<dyn AnyViewController>>) {
        self.finish_transition();

        if self.entries.is_empty() {
            let id = self.insert(controller.into());
            self.entry_mut(id).controller.did_appear();
            return;
        }

        let visible_before = self.visible_ids(&[]);
        let outgoing = self.top_id();
        let removed = self.entries.iter().map(|entry| entry.id).collect();
        let incoming = self.insert(controller.into());
        self.begin(
            NavigationOperation::SetRoot,
            outgoing,
            incoming,
            visible_before,
            removed,
        );
    }

    pub fn advance(&mut self, delta_seconds: f32) {
        self.deliver_messages();
        self.apply_requests();

        let Some(active) = &mut self.transition else {
            return;
        };
        active.elapsed += delta_seconds;
        let progress = active.progress();
        active.transition.progressed(progress);

        if progress >= 1.0 {
            self.finish_transition();
        }
    }

    // The view controller on top once any running transition ends.
    pub fn top(&self) -> &dyn AnyViewController {
        let removed = self.removed_ids();
        let entry = self
            .entries
            .iter()
            .rev()
            .find(|entry| !removed.contains(&entry.id))
            .expect("the stack always holds a root");

        entry.controller.as_ref()
    }

    pub fn is_transitioning(&self) -> bool {
        self.transition.is_some()
    }

    // Every view controller's view, drawn or not, so each keeps its state while it stays in the stack. Only the
    // top one takes pointer events, and none does during a transition.
    pub fn view(&self) -> impl View + use<> {
        let removed = self.removed_ids();
        let mut drawn = self.visible_ids(&removed);

        if let Some(active) = &self.transition {
            drawn.extend(active.visible_before.iter().copied());
        }

        let top = self.top_id_excluding(&removed);
        let layers: Vec<_> = self
            .entries
            .iter()
            .map(|entry| {
                let appearance = self.appearance(entry.id);
                let interactive = self.transition.is_none() && entry.id == top;
                let fill = Some(f32::INFINITY);
                let navigation = self.navigation();
                let id = entry.id;
                let dismiss = Action::new(move |()| navigation.dismiss(id));
                let view = Visibility::new(
                    entry
                        .controller
                        .root_view(entry.id, &self.messages)
                        .environment::<DismissKey>(dismiss)
                        .opacity(appearance.opacity)
                        .offset(appearance.offset),
                    drawn.contains(&entry.id),
                    interactive,
                )
                .max_frame(fill, fill, Alignment::CENTER);

                NavigationLayer { id: entry.id, view }
            })
            .collect();

        ZStack::new(ForEach::with_id(layers, |layer| layer.id, |layer| layer.view))
    }

    fn deliver_messages(&mut self) {
        let messages = std::mem::take(&mut *self.messages.borrow_mut());

        for message in messages {
            if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == message.id) {
                (message.call)(entry.controller.as_mut() as &mut dyn Any);
            }
        }
    }

    fn apply_requests(&mut self) {
        let requests = std::mem::take(&mut *self.requests.borrow_mut());

        for request in requests {
            match request {
                NavigationRequest::Push(controller) => self.push(controller),
                NavigationRequest::Pop => {
                    self.pop();
                }
                NavigationRequest::SetRoot(controller) => self.set_root(controller),
                NavigationRequest::Dismiss { id } => self.dismiss(id),
            }
        }
    }

    // Pops the view controller when it is on top; otherwise removes it from under the top at once.
    fn dismiss(&mut self, id: u64) {
        self.finish_transition();

        if self.entries.len() < 2 || !self.entries.iter().any(|entry| entry.id == id) {
            return;
        }

        if id == self.top_id() {
            self.pop();
            return;
        }

        let visible_before = self.visible_ids(&[]);
        let index = self
            .entries
            .iter()
            .position(|entry| entry.id == id)
            .expect("the entry was found above");
        let mut entry = self.entries.remove(index);

        if visible_before.contains(&id) {
            entry.controller.did_disappear();
        }

        entry.controller.did_unload();
    }

    fn entry_mut(&mut self, id: u64) -> &mut NavigationEntry {
        self.entries
            .iter_mut()
            .find(|entry| entry.id == id)
            .expect("the entry is in the stack")
    }

    fn appearance(&self, id: u64) -> TransitionAppearance {
        let Some(active) = &self.transition else {
            return TransitionAppearance::IDENTITY;
        };
        let progress = active.progress();

        if id == active.incoming {
            active.transition.appearance(progress, TransitionRole::Incoming)
        } else if id == active.outgoing {
            active.transition.appearance(progress, TransitionRole::Outgoing)
        } else {
            TransitionAppearance::IDENTITY
        }
    }

    fn insert(&mut self, mut controller: Box<dyn AnyViewController>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        controller.did_load();
        self.entries.push(NavigationEntry { id, controller });
        id
    }

    fn begin(
        &mut self,
        operation: NavigationOperation,
        outgoing: u64,
        incoming: u64,
        visible_before: Vec<u64>,
        removed: Vec<u64>,
    ) {
        let transition = self
            .delegate
            .transition(operation, self.controller(outgoing), self.controller(incoming));
        self.transition = Some(ActiveTransition {
            transition,
            elapsed: 0.0,
            outgoing,
            incoming,
            visible_before,
            removed,
        });

        if self.transition.as_ref().is_some_and(|active| active.progress() >= 1.0) {
            self.finish_transition();
        }
    }

    // Removes what the transition removes and tells each controller whether it appeared or disappeared, the
    // disappearances first, as UIKit does.
    fn finish_transition(&mut self) {
        let Some(mut active) = self.transition.take() else {
            return;
        };
        active.transition.progressed(1.0);
        let (removed, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut self.entries)
            .into_iter()
            .partition(|entry| active.removed.contains(&entry.id));
        self.entries = kept;
        let visible_after = self.visible_ids(&[]);
        let was_visible = |entry: &NavigationEntry| active.visible_before.contains(&entry.id);

        for entry in &mut self.entries {
            if was_visible(entry) && !visible_after.contains(&entry.id) {
                entry.controller.did_disappear();
            }
        }

        for mut entry in removed {
            if was_visible(&entry) {
                entry.controller.did_disappear();
            }

            entry.controller.did_unload();
        }

        for entry in &mut self.entries {
            if visible_after.contains(&entry.id) && !was_visible(entry) {
                entry.controller.did_appear();
            }
        }
    }

    // The view controllers drawn when the stack holds every entry but `excluded`: from the top down to the first
    // full-screen one.
    fn visible_ids(&self, excluded: &[u64]) -> Vec<u64> {
        let mut visible = Vec::new();

        for entry in self.entries.iter().rev().filter(|entry| !excluded.contains(&entry.id)) {
            visible.push(entry.id);

            if entry.controller.presentation() == Presentation::FullScreen {
                break;
            }
        }

        visible
    }

    fn removed_ids(&self) -> Vec<u64> {
        self.transition
            .as_ref()
            .map_or_else(Vec::new, |active| active.removed.clone())
    }

    fn top_id(&self) -> u64 {
        self.top_id_excluding(&[])
    }

    fn top_id_excluding(&self, excluded: &[u64]) -> u64 {
        self.entries
            .iter()
            .rev()
            .find(|entry| !excluded.contains(&entry.id))
            .expect("the stack always holds a root")
            .id
    }

    fn controller(&self, id: u64) -> &dyn AnyViewController {
        self.entries
            .iter()
            .find(|entry| entry.id == id)
            .expect("transitions only name entries in the stack")
            .controller
            .as_ref()
    }
}
