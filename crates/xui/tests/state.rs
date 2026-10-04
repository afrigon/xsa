use std::cell::RefCell;
use std::rc::Rc;

use xui::{Binding, Context, EmptyView, Environment, Interface, Size, View};

const VIEWPORT: Size = Size {
    width: 400.0,
    height: 300.0,
};

#[derive(Clone, PartialEq, Debug)]
struct Seen {
    count: u32,
    label: String,
}

// What a view saw in its body each frame, and a binding to its state for the test to write through.
#[derive(Clone, Default)]
struct Probe {
    seen: Rc<RefCell<Vec<Seen>>>,
    count: Rc<RefCell<Option<Binding<u32>>>>,
}

struct Counter {
    probe: Probe,
}

impl View for Counter {
    fn body(&self, context: &Context) -> impl View {
        let count = context.state(|| 0_u32);
        let label = context.state(|| "start".to_string());
        self.probe.seen.borrow_mut().push(Seen {
            count: count.get(),
            label: label.get(),
        });
        *self.probe.count.borrow_mut() = Some(count.binding());

        EmptyView
    }
}

fn render(interface: &mut Interface, view: &impl View) {
    interface.render(view, VIEWPORT, &Environment::default()).unwrap();
}

fn seen_counts(probe: &Probe) -> Vec<u32> {
    probe.seen.borrow().iter().map(|seen| seen.count).collect()
}

fn set_count(probe: &Probe, value: u32) {
    probe.count.borrow().as_ref().unwrap().set(value);
}

#[test]
fn state_survives_frames_and_bindings_write_it() {
    let probe = Probe::default();
    let mut interface = Interface::new();
    render(&mut interface, &Counter { probe: probe.clone() });
    set_count(&probe, 5);
    render(&mut interface, &Counter { probe: probe.clone() });
    render(&mut interface, &Counter { probe: probe.clone() });

    assert_eq!(seen_counts(&probe), [0, 5, 5]);
}

#[test]
fn each_call_site_has_its_own_state() {
    let probe = Probe::default();
    let mut interface = Interface::new();
    render(&mut interface, &Counter { probe: probe.clone() });
    set_count(&probe, 3);
    render(&mut interface, &Counter { probe: probe.clone() });

    assert_eq!(
        probe.seen.borrow().last().unwrap(),
        &Seen {
            count: 3,
            label: "start".to_string(),
        }
    );
}

#[test]
fn a_new_identity_starts_with_fresh_state() {
    let probe = Probe::default();
    let mut interface = Interface::new();
    let counter = |id: u32| Counter { probe: probe.clone() }.id(id);
    render(&mut interface, &counter(1));
    set_count(&probe, 7);
    render(&mut interface, &counter(1));
    render(&mut interface, &counter(2));

    assert_eq!(seen_counts(&probe), [0, 7, 0]);
}

struct Parent {
    seen: Rc<RefCell<Vec<u32>>>,
}

struct Incrementer {
    value: Binding<u32>,
}

impl View for Parent {
    fn body(&self, context: &Context) -> impl View {
        let count = context.state(|| 0_u32);
        self.seen.borrow_mut().push(count.get());

        Incrementer { value: count.binding() }
    }
}

impl View for Incrementer {
    fn body(&self, _context: &Context) -> impl View {
        self.value.set(self.value.get() + 1);

        EmptyView
    }
}

#[test]
fn children_edit_their_parents_state_through_bindings() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let mut interface = Interface::new();

    for _ in 0..3 {
        render(&mut interface, &Parent { seen: seen.clone() });
    }

    assert_eq!(*seen.borrow(), [0, 1, 2]);
}
