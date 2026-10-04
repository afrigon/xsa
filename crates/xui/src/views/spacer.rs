use crate::layout::SpacerLayout;
use crate::{Context, Never, Node, UpdateContext, View};

// Takes the space offered along its stack's axis, so the views around it are pushed apart. Outside a stack it
// takes the space offered on both axes.
#[derive(Default)]
pub struct Spacer {
    min_length: f32,
}

impl Spacer {
    pub fn new() -> Spacer {
        Spacer::default()
    }

    pub fn min_length(self, min_length: f32) -> Spacer {
        Spacer { min_length }
    }
}

impl View for Spacer {
    fn body(&self, _context: &Context) -> impl View {
        Never::primitive_body()
    }

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        node.set_layout(SpacerLayout {
            min_length: self.min_length,
        });
        node.update_children(&[], context)
    }
}
