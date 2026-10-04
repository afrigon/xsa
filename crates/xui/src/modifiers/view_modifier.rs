use crate::{Context, ModifierContent, View};

// A reusable modification of any view, like SwiftUI's `ViewModifier`: `body` builds on `content`, which stands
// for the modified view, and can read the context.
pub trait ViewModifier: 'static {
    fn body(&self, content: ModifierContent, context: &Context) -> impl View;
}
