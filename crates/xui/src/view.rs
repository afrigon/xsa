use crate::{
    DrawList, EdgeInsets, Environment, EnvironmentKey, EnvironmentModifier, Font, FontKey, ForegroundStyleKey,
    LinearColor, ModifiedContent, Padding, Rect, Size, SizeProposal, ViewContext, ViewModifier,
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

    fn padding(self, insets: EdgeInsets) -> Padding<Self>
    where
        Self: Sized,
    {
        Padding::new(self, insets)
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
}
