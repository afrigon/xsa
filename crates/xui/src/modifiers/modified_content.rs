use crate::{Environment, View, ViewModifier};

pub struct ModifiedContent<Content: View, Modifier: ViewModifier> {
    content: Content,
    modifier: Modifier,
}

impl<Content: View, Modifier: ViewModifier> ModifiedContent<Content, Modifier> {
    pub fn new(content: Content, modifier: Modifier) -> ModifiedContent<Content, Modifier> {
        ModifiedContent { content, modifier }
    }
}

impl<Content: View, Modifier: ViewModifier> View for ModifiedContent<Content, Modifier> {
    fn body(&self, environment: &Environment) -> impl View {
        self.modifier.body(&self.content, environment)
    }
}
