use std::cell::{Cell, RefCell};
use std::rc::Rc;

use xui::{
    Alignment, AnyView, Context, DrawList, Either, EmptyView, Environment, ForEach, HStack, HorizontalAlignment,
    Interface, LayoutContext, Node, NodeLayout, Point, Rect, Size, SizeProposal, Spacer, UpdateContext, VStack, View,
    ZStack,
};

const VIEWPORT: Size = Size {
    width: 400.0,
    height: 300.0,
};
const SPACING: f32 = 8.0;

type Placements = Rc<RefCell<Vec<Rect>>>;

// Counts what the tree does with blocks: layouts created and sizes measured.
#[derive(Clone, Default)]
struct Probe {
    placements: Placements,
    layouts_created: Rc<Cell<usize>>,
    measurements: Rc<Cell<usize>>,
}

// A fixed-size primitive that records what the tree does with it.
struct Block {
    size: Size,
    probe: Probe,
}

impl Block {
    fn new(width: f32, height: f32, probe: &Probe) -> Block {
        Block {
            size: Size { width, height },
            probe: probe.clone(),
        }
    }
}

struct BlockLayout {
    size: Size,
    probe: Probe,
}

impl NodeLayout for BlockLayout {
    fn size_that_fits(
        &mut self,
        _proposal: SizeProposal,
        _children: &mut [Node],
        _context: &mut LayoutContext,
    ) -> anyhow::Result<Size> {
        self.probe.measurements.set(self.probe.measurements.get() + 1);
        Ok(self.size)
    }

    fn place(
        &mut self,
        bounds: Rect,
        _children: &mut [Node],
        _context: &mut LayoutContext,
        _draw_list: &mut DrawList,
    ) -> anyhow::Result<()> {
        self.probe.placements.borrow_mut().push(bounds);
        Ok(())
    }
}

impl View for Block {
    fn body(&self, _context: &Context) -> impl View {
        EmptyView
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let layout = node.layout_mut(|| {
            self.probe.layouts_created.set(self.probe.layouts_created.get() + 1);
            BlockLayout {
                size: self.size,
                probe: self.probe.clone(),
            }
        });
        let resized = layout.size != self.size;
        layout.size = self.size;

        if resized {
            node.mark_changed();
        }

        node.update_children(&[], context)
    }
}

fn render(view: &impl View) {
    Interface::new()
        .render(view, VIEWPORT, &Environment::default())
        .unwrap();
}

fn origins(probe: &Probe) -> Vec<Point> {
    probe.placements.borrow().iter().map(|rect| rect.origin).collect()
}

fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}

#[test]
fn vertical_stacks_space_and_align_their_subviews() {
    let probe = Probe::default();
    let stack = VStack::new((Block::new(100.0, 20.0, &probe), Block::new(50.0, 30.0, &probe)))
        .alignment(HorizontalAlignment::Leading)
        .max_frame(Some(f32::INFINITY), Some(f32::INFINITY), Alignment::TOP_LEADING);
    render(&stack);

    assert_eq!(origins(&probe), [point(0.0, 0.0), point(0.0, 20.0 + SPACING)]);
}

#[test]
fn horizontal_stacks_center_across_and_spacers_push_apart() {
    let probe = Probe::default();
    let stack = HStack::new((
        Block::new(100.0, 20.0, &probe),
        Spacer::new(),
        Block::new(50.0, 40.0, &probe),
    ))
    .max_frame(Some(f32::INFINITY), Some(f32::INFINITY), Alignment::TOP);
    render(&stack);

    assert_eq!(origins(&probe), [point(0.0, 10.0), point(350.0, 0.0)]);
}

#[test]
fn absent_views_take_no_space_or_spacing() {
    let probe = Probe::default();
    let missing: Option<Block> = None;
    let stack = VStack::new((Block::new(10.0, 10.0, &probe), missing, Block::new(10.0, 10.0, &probe))).max_frame(
        None,
        Some(f32::INFINITY),
        Alignment::TOP,
    );
    render(&stack);

    assert_eq!(probe.placements.borrow()[1].origin.y, 10.0 + SPACING);
}

#[test]
fn for_each_lays_out_one_view_per_item() {
    let probe = Probe::default();
    let stack = VStack::new(ForEach::new(0..3_u32, |index| {
        Block::new(10.0, 10.0 + index as f32, &probe)
    }))
    .spacing(0.0)
    .max_frame(None, Some(f32::INFINITY), Alignment::TOP);
    render(&stack);

    let tops: Vec<f32> = origins(&probe).iter().map(|origin| origin.y).collect();
    assert_eq!(tops, [0.0, 10.0, 21.0]);
}

#[test]
fn z_stacks_overlay_and_align() {
    let probe = Probe::default();
    let stack = ZStack::new((
        Block::new(200.0, 100.0, &probe),
        AnyView::new(Block::new(20.0, 10.0, &probe)),
    ))
    .alignment(Alignment::BOTTOM_TRAILING)
    .max_frame(Some(f32::INFINITY), Some(f32::INFINITY), Alignment::TOP_LEADING);
    render(&stack);

    assert_eq!(origins(&probe), [point(0.0, 0.0), point(180.0, 90.0)]);
}

#[test]
fn frames_fix_or_fill_and_align_their_content() {
    let probe = Probe::default();
    let view = ZStack::new((
        Block::new(10.0, 10.0, &probe).frame(Some(100.0), Some(50.0), Alignment::BOTTOM_TRAILING),
        Block::new(10.0, 10.0, &probe).max_frame(Some(f32::INFINITY), Some(f32::INFINITY), Alignment::TRAILING),
    ));
    render(&view);

    let origins = origins(&probe);
    assert_eq!(origins[1], point(390.0, 145.0));
    assert_eq!(origins[0], point(150.0 + 90.0, 125.0 + 40.0));
}

fn render_with(interface: &mut Interface, view: &impl View) {
    interface.render(view, VIEWPORT, &Environment::default()).unwrap();
}

fn column(sizes: &[f32], probe: &Probe) -> impl View + use<> {
    VStack::new(
        sizes
            .iter()
            .map(|height| Block::new(10.0, *height, probe))
            .collect::<Vec<_>>(),
    )
}

#[test]
fn unchanged_views_are_not_measured_again() {
    let probe = Probe::default();
    let mut interface = Interface::new();
    render_with(&mut interface, &column(&[10.0, 20.0, 30.0], &probe));
    let first = probe.measurements.get();
    assert!(first > 0);

    render_with(&mut interface, &column(&[10.0, 20.0, 30.0], &probe));
    assert_eq!(probe.measurements.get(), first);
}

#[test]
fn a_changed_view_is_measured_again_and_its_siblings_are_not() {
    let probe = Probe::default();
    let mut interface = Interface::new();
    render_with(&mut interface, &column(&[10.0, 20.0, 30.0], &probe));
    let before = probe.measurements.get();

    render_with(&mut interface, &column(&[10.0, 25.0, 30.0], &probe));
    let remeasured = probe.measurements.get() - before;
    assert!(remeasured > 0 && remeasured < before, "{remeasured} of {before}");
    assert_eq!(probe.layouts_created.get(), 3);
}

#[test]
fn for_each_keeps_nodes_when_items_move() {
    let probe = Probe::default();
    let mut interface = Interface::new();
    let list = |ids: &[u32]| {
        VStack::new(ForEach::new(ids.to_vec(), |id| {
            Block::new(10.0, 10.0 + id as f32, &probe)
        }))
    };
    render_with(&mut interface, &list(&[1, 2, 3]));
    render_with(&mut interface, &list(&[3, 1, 2]));
    render_with(&mut interface, &list(&[3, 2]));

    assert_eq!(probe.layouts_created.get(), 3);
}

#[test]
fn switching_branches_and_ids_starts_over() {
    let probe = Probe::default();
    let mut interface = Interface::new();
    let branch = |first: bool| -> Either<Block, VStack<Block>> {
        if first {
            Either::First(Block::new(10.0, 10.0, &probe))
        } else {
            Either::Second(VStack::new(Block::new(10.0, 10.0, &probe)))
        }
    };
    render_with(&mut interface, &branch(true));
    render_with(&mut interface, &branch(true));
    assert_eq!(probe.layouts_created.get(), 1);
    render_with(&mut interface, &branch(false));
    assert_eq!(probe.layouts_created.get(), 2);

    let identified = |id: u32| Block::new(10.0, 10.0, &probe).id(id);
    render_with(&mut interface, &identified(1));
    render_with(&mut interface, &identified(1));
    assert_eq!(probe.layouts_created.get(), 3);
    render_with(&mut interface, &identified(2));
    assert_eq!(probe.layouts_created.get(), 4);
}
