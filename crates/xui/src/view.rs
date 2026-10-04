use crate::{
    Alignment, DrawList, EdgeInsets, Environment, EnvironmentKey, EnvironmentModifier, FixedFrame, Font, FontKey,
    ForegroundStyleKey, LinearColor, MaxFrame, ModifiedContent, Padding, Rect, Size, SizeProposal, Subview,
    ViewContext, ViewModifier,
};

// A view describes its content in `body`, like SwiftUI. Layout follows SwiftUI too: a parent proposes a size,
// the view answers with the size it takes, and the parent places it; sizes and positions are in logical points.
// Primitive views override `size_that_fits` and `place` instead of having a body.
pub trait View {
    fn body(&self, environment: &Environment) -> impl View;

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        let environment = context.environment.clone();
        self.body(&environment).size_that_fits(proposal, context)
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        let environment = context.environment.clone();
        self.body(&environment).place(bounds, context, draw_list)
    }

    // The views a stack lays out for this one: itself, or for lists (tuples, Option, ForEach…) their items,
    // flattened, so an absent item takes neither space nor spacing.
    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<&'a dyn Subview>)
    where
        Self: Sized,
    {
        subviews.push(self);
    }

    fn padding(self, insets: EdgeInsets) -> Padding<Self>
    where
        Self: Sized,
    {
        Padding::new(self, insets)
    }

    // A fixed size on either axis; an axis left as None keeps the content's size.
    fn frame(self, width: Option<f32>, height: Option<f32>, alignment: Alignment) -> FixedFrame<Self>
    where
        Self: Sized,
    {
        FixedFrame::new(self, width, height, alignment)
    }

    // Grows to the proposed size up to the maximum (f32::INFINITY fills it); an axis left as None hugs the content.
    fn max_frame(self, max_width: Option<f32>, max_height: Option<f32>, alignment: Alignment) -> MaxFrame<Self>
    where
        Self: Sized,
    {
        MaxFrame::new(self, max_width, max_height, alignment)
    }

    fn modifier<Modifier: ViewModifier>(self, modifier: Modifier) -> ModifiedContent<Self, Modifier>
    where
        Self: Sized,
    {
        ModifiedContent::new(self, modifier)
    }

    fn environment<Key: EnvironmentKey>(self, value: Key::Value) -> EnvironmentModifier<Self, Key>
    where
        Self: Sized,
    {
        EnvironmentModifier::new(self, value)
    }

    fn font(self, font: Option<Font>) -> EnvironmentModifier<Self, FontKey>
    where
        Self: Sized,
    {
        self.environment::<FontKey>(font)
    }

    fn foreground_style(self, color: LinearColor) -> EnvironmentModifier<Self, ForegroundStyleKey>
    where
        Self: Sized,
    {
        self.environment::<ForegroundStyleKey>(color)
    }
}

impl<Content: View> View for &Content {
    fn body(&self, environment: &Environment) -> impl View {
        (**self).body(environment)
    }

    fn size_that_fits(&self, proposal: SizeProposal, context: &mut ViewContext) -> anyhow::Result<Size> {
        (**self).size_that_fits(proposal, context)
    }

    fn place(&self, bounds: Rect, context: &mut ViewContext, draw_list: &mut DrawList) -> anyhow::Result<()> {
        (**self).place(bounds, context, draw_list)
    }

    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<&'a dyn Subview>) {
        (**self).collect_subviews(subviews);
    }
}
