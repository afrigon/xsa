use crate::{DrawList, Environment, Rect, Size, SizeProposal, View, ViewContext};

// The body of primitive views, like SwiftUI's `Never`: they lay themselves out, so their body is never asked for.
#[derive(Clone, Copy)]
pub enum Never {}

impl Never {
    pub(crate) fn primitive_body() -> Never {
        unreachable!("primitive views lay themselves out and have no body")
    }
}

impl View for Never {
    fn body(&self, _environment: &Environment) -> impl View {
        *self
    }

    fn size_that_fits(&self, _proposal: SizeProposal, _context: &mut ViewContext) -> anyhow::Result<Size> {
        match *self {}
    }

    fn place(&self, _bounds: Rect, _context: &mut ViewContext, _draw_list: &mut DrawList) -> anyhow::Result<()> {
        match *self {}
    }
}
