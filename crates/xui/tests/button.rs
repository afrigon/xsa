use std::cell::{Cell, RefCell};
use std::rc::Rc;

use xui::{
    Alignment, Button, ButtonConfiguration, ButtonStyle, Context, DrawList, EmptyView, Environment, Interface,
    LayoutContext, ModifierContent, Node, NodeLayout, Point, PointerButton, PointerEvent, PointerEventKind, Rect,
    ScaleFactorKey, Size, SizeProposal, UpdateContext, View, ZStack,
};

const VIEWPORT: Size = Size {
    width: 400.0,
    height: 300.0,
};

// A 100 × 40 label at the top-left of the viewport.
struct Label;

struct LabelLayout;

impl NodeLayout for LabelLayout {
    fn size_that_fits(
        &mut self,
        _proposal: SizeProposal,
        _children: &mut [Node],
        _context: &mut LayoutContext,
    ) -> anyhow::Result<Size> {
        Ok(Size {
            width: 100.0,
            height: 40.0,
        })
    }

    fn place(
        &mut self,
        _bounds: Rect,
        _children: &mut [Node],
        _context: &mut LayoutContext,
        _draw_list: &mut DrawList,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

impl View for Label {
    fn body(&self, _context: &Context) -> impl View {
        EmptyView
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.layout_mut(|| LabelLayout);
        node.update_children(&[], context)
    }
}

// Records the configuration each button was drawn with.
struct RecordingStyle {
    seen: Rc<RefCell<Vec<ButtonConfiguration>>>,
}

impl ButtonStyle for RecordingStyle {
    fn body(&self, label: ModifierContent, configuration: ButtonConfiguration, _context: &Context) -> impl View {
        self.seen.borrow_mut().push(configuration);
        label
    }
}

fn counting_button(clicks: &Rc<Cell<u32>>) -> impl View + use<> {
    let clicks = clicks.clone();
    Button::new(Label, move || clicks.set(clicks.get() + 1)).max_frame(
        Some(f32::INFINITY),
        Some(f32::INFINITY),
        Alignment::TOP_LEADING,
    )
}

fn render(interface: &mut Interface, view: &impl View, environment: &Environment) {
    interface.render(view, VIEWPORT, environment).unwrap();
}

fn event(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        kind,
        position: Point { x, y },
    }
}

const PRESS: PointerEventKind = PointerEventKind::Pressed {
    button: PointerButton::Primary,
};
const RELEASE: PointerEventKind = PointerEventKind::Released {
    button: PointerButton::Primary,
};

#[test]
fn a_click_inside_runs_the_action_and_releasing_outside_cancels() {
    let clicks = Rc::new(Cell::new(0));
    let mut interface = Interface::new();
    render(&mut interface, &counting_button(&clicks), &Environment::default());

    assert!(interface.handle_event(event(PRESS, 50.0, 20.0)));
    assert!(interface.handle_event(event(RELEASE, 50.0, 20.0)));
    assert_eq!(clicks.get(), 1);

    assert!(interface.handle_event(event(PRESS, 50.0, 20.0)));
    assert!(interface.handle_event(event(RELEASE, 300.0, 200.0)));
    assert_eq!(clicks.get(), 1);

    assert!(!interface.handle_event(event(PRESS, 300.0, 200.0)));
    assert!(!interface.handle_event(event(RELEASE, 50.0, 20.0)));
    assert_eq!(clicks.get(), 1);
}

#[test]
fn styles_see_hover_and_press() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let clicks = Rc::new(Cell::new(0));
    let mut interface = Interface::new();
    let view = || counting_button(&clicks).button_style(RecordingStyle { seen: seen.clone() });
    let environment = Environment::default();
    render(&mut interface, &view(), &environment);
    interface.handle_event(event(PointerEventKind::Moved, 10.0, 10.0));
    render(&mut interface, &view(), &environment);
    interface.handle_event(event(PRESS, 10.0, 10.0));
    render(&mut interface, &view(), &environment);
    interface.handle_event(event(PointerEventKind::Left, 0.0, 0.0));
    interface.handle_event(event(RELEASE, 0.0, 0.0));
    render(&mut interface, &view(), &environment);

    let hovered = |is_hovered, is_pressed| ButtonConfiguration { is_hovered, is_pressed };
    assert_eq!(
        *seen.borrow(),
        [
            hovered(false, false),
            hovered(true, false),
            hovered(true, true),
            hovered(false, false)
        ]
    );
}

#[test]
fn only_the_topmost_button_takes_the_pointer() {
    let below = Rc::new(Cell::new(0));
    let above = Rc::new(Cell::new(0));
    let mut interface = Interface::new();
    render(
        &mut interface,
        &ZStack::new((counting_button(&below), counting_button(&above))),
        &Environment::default(),
    );

    interface.handle_event(event(PRESS, 50.0, 20.0));
    interface.handle_event(event(RELEASE, 50.0, 20.0));

    assert_eq!((below.get(), above.get()), (0, 1));
}

#[test]
fn positions_are_converted_from_physical_pixels() {
    let clicks = Rc::new(Cell::new(0));
    let mut interface = Interface::new();
    let doubled = Environment::default().with::<ScaleFactorKey>(2.0);
    render(&mut interface, &counting_button(&clicks), &doubled);

    assert!(!interface.handle_event(event(PRESS, 250.0, 20.0)));
    assert!(interface.handle_event(event(PRESS, 190.0, 70.0)));
    interface.handle_event(event(RELEASE, 190.0, 70.0));
    assert_eq!(clicks.get(), 1);
}
