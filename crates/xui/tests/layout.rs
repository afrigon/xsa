use std::cell::RefCell;
use std::rc::Rc;

use xui::{
    Alignment, AnyView, DrawList, EmptyView, Environment, ForEach, HStack, HorizontalAlignment, Interface, Point, Rect,
    Size, SizeProposal, Spacer, VStack, View, ViewContext, ZStack,
};

const VIEWPORT: Size = Size {
    width: 400.0,
    height: 300.0,
};
const SPACING: f32 = 8.0;

type Placements = Rc<RefCell<Vec<Rect>>>;

// A fixed-size view that records where it is placed.
struct Block {
    size: Size,
    placements: Placements,
}

impl Block {
    fn new(width: f32, height: f32, placements: &Placements) -> Block {
        Block {
            size: Size { width, height },
            placements: placements.clone(),
        }
    }
}

impl View for Block {
    fn body(&self, _environment: &Environment) -> impl View {
        EmptyView
    }

    fn size_that_fits(&self, _proposal: SizeProposal, _context: &mut ViewContext) -> anyhow::Result<Size> {
        Ok(self.size)
    }

    fn place(&self, bounds: Rect, _context: &mut ViewContext, _draw_list: &mut DrawList) -> anyhow::Result<()> {
        self.placements.borrow_mut().push(bounds);
        Ok(())
    }
}

fn render(view: &impl View) {
    Interface::new()
        .render(view, VIEWPORT, &Environment::default())
        .unwrap();
}

fn origins(placements: &Placements) -> Vec<Point> {
    placements.borrow().iter().map(|rect| rect.origin).collect()
}

fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}

#[test]
fn vertical_stacks_space_and_align_their_subviews() {
    let placements = Placements::default();
    let stack = VStack::new((
        Block::new(100.0, 20.0, &placements),
        Block::new(50.0, 30.0, &placements),
    ))
    .alignment(HorizontalAlignment::Leading)
    .max_frame(Some(f32::INFINITY), Some(f32::INFINITY), Alignment::TOP_LEADING);
    render(&stack);

    assert_eq!(origins(&placements), [point(0.0, 0.0), point(0.0, 20.0 + SPACING)]);
}

#[test]
fn horizontal_stacks_center_across_and_spacers_push_apart() {
    let placements = Placements::default();
    let stack = HStack::new((
        Block::new(100.0, 20.0, &placements),
        Spacer::new(),
        Block::new(50.0, 40.0, &placements),
    ))
    .max_frame(Some(f32::INFINITY), Some(f32::INFINITY), Alignment::TOP);
    render(&stack);

    assert_eq!(origins(&placements), [point(0.0, 10.0), point(350.0, 0.0)]);
}

#[test]
fn absent_views_take_no_space_or_spacing() {
    let placements = Placements::default();
    let missing: Option<Block> = None;
    let stack = VStack::new((
        Block::new(10.0, 10.0, &placements),
        missing,
        Block::new(10.0, 10.0, &placements),
    ))
    .max_frame(None, Some(f32::INFINITY), Alignment::TOP);
    render(&stack);

    assert_eq!(placements.borrow()[1].origin.y, 10.0 + SPACING);
}

#[test]
fn for_each_lays_out_one_view_per_item() {
    let placements = Placements::default();
    let stack = VStack::new(ForEach::new(0..3_u32, |index| {
        Block::new(10.0, 10.0 + index as f32, &placements)
    }))
    .spacing(0.0)
    .max_frame(None, Some(f32::INFINITY), Alignment::TOP);
    render(&stack);

    let tops: Vec<f32> = origins(&placements).iter().map(|origin| origin.y).collect();
    assert_eq!(tops, [0.0, 10.0, 21.0]);
}

#[test]
fn z_stacks_overlay_and_align() {
    let placements = Placements::default();
    let stack = ZStack::new((
        Block::new(200.0, 100.0, &placements),
        AnyView::new(Block::new(20.0, 10.0, &placements)),
    ))
    .alignment(Alignment::BOTTOM_TRAILING)
    .max_frame(Some(f32::INFINITY), Some(f32::INFINITY), Alignment::TOP_LEADING);
    render(&stack);

    assert_eq!(origins(&placements), [point(0.0, 0.0), point(180.0, 90.0)]);
}

#[test]
fn frames_fix_or_fill_and_align_their_content() {
    let placements = Placements::default();
    let view = ZStack::new((
        Block::new(10.0, 10.0, &placements).frame(Some(100.0), Some(50.0), Alignment::BOTTOM_TRAILING),
        Block::new(10.0, 10.0, &placements).max_frame(Some(f32::INFINITY), Some(f32::INFINITY), Alignment::TRAILING),
    ));
    render(&view);

    let origins = origins(&placements);
    assert_eq!(origins[1], point(390.0, 145.0));
    assert_eq!(origins[0], point(150.0 + 90.0, 125.0 + 40.0));
}
