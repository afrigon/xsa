use std::hash::Hash;
use std::rc::Rc;

use crate::{
    Alignment, ButtonStyle, ButtonStyleKey, Context, EdgeInsets, EnvironmentKey, EnvironmentModifier, FixedFrame, Font,
    FontKey, ForegroundStyleKey, IdentifiedView, LinearColor, MaxFrame, ModifiedContent, Node, Offset, OpacityModifier,
    Padding, PassthroughLayout, Point, SubviewEntry, UpdateContext, ViewModifier, Visibility,
};

// A view describes its content in `body`, like SwiftUI. Views are values rebuilt every frame and own their data;
// what must outlive a frame (shaped text, measured sizes) lives in the view's node. Layout follows SwiftUI: a
// parent proposes a size, the view answers with the size it takes, and the parent places it, in logical points.
// Primitive views override `update` to keep their inputs in a node layout instead of having a body.
pub trait View: 'static {
    fn body(&self, context: &Context) -> impl View;

    fn update(&self, node: &mut Node, context: &mut UpdateContext) -> anyhow::Result<()> {
        let body = self.body(&node.context(&context.environment));
        node.layout_mut(|| PassthroughLayout);
        node.update_children(&[SubviewEntry::new(&body)], context)
    }

    // The views a stack lays out for this one: itself, or for lists (tuples, Option, ForEach…) their items,
    // flattened, so an absent item takes neither space nor spacing.
    fn collect_subviews<'a>(&'a self, subviews: &mut Vec<SubviewEntry<'a>>)
    where
        Self: Sized,
    {
        subviews.push(SubviewEntry::new(self));
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

    // Multiplies the opacity of the view and everything inside it.
    fn opacity(self, opacity: f32) -> OpacityModifier<Self>
    where
        Self: Sized,
    {
        OpacityModifier::new(self, opacity)
    }

    // Draws the view shifted by `offset`, keeping its place in the layout, like SwiftUI's `.offset()`.
    fn offset(self, offset: Point) -> Offset<Self>
    where
        Self: Sized,
    {
        Offset::new(self, offset)
    }

    // A hidden view keeps its place in the layout and its state, but is neither drawn nor hit-tested.
    fn hidden(self, hidden: bool) -> Visibility<Self>
    where
        Self: Sized,
    {
        Visibility::new(self, !hidden, true)
    }

    // Whether pointer events reach the view and everything inside it, like SwiftUI's `.allowsHitTesting()`.
    fn allows_hit_testing(self, allowed: bool) -> Visibility<Self>
    where
        Self: Sized,
    {
        Visibility::new(self, true, allowed)
    }

    // A new identity whenever `id` changes, so the view starts over with fresh state.
    fn id(self, id: impl Hash) -> IdentifiedView<Self>
    where
        Self: Sized,
    {
        IdentifiedView::new(self, id)
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

    // The style of every button in this view, like SwiftUI's `.buttonStyle()`.
    fn button_style(self, style: impl ButtonStyle) -> EnvironmentModifier<Self, ButtonStyleKey>
    where
        Self: Sized,
    {
        self.environment::<ButtonStyleKey>(Rc::new(style))
    }

    fn foreground_style(self, color: LinearColor) -> EnvironmentModifier<Self, ForegroundStyleKey>
    where
        Self: Sized,
    {
        self.environment::<ForegroundStyleKey>(color)
    }
}
