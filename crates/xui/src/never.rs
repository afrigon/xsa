use crate::{Context, Node, UpdateContext, View};

// The body of primitive views, like SwiftUI's `Never`: they update their node themselves, so their body is never
// asked for.
#[derive(Clone, Copy)]
pub enum Never {}

impl Never {
    pub(crate) fn primitive_body() -> Never {
        unreachable!("primitive views update their node themselves and have no body")
    }
}

impl View for Never {
    fn body(&self, _context: &Context) -> impl View {
        *self
    }

    fn update(&self, _node: &mut Node, _context: &mut UpdateContext) -> anyhow::Result<()> {
        match *self {}
    }
}
