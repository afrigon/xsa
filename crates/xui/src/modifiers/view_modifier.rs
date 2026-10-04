use crate::{Environment, View};

// A reusable modification of any view, like SwiftUI's `ViewModifier`: `body` receives the modified view and can
// read the environment.
pub trait ViewModifier {
    fn body<'a, Content: View>(&'a self, content: &'a Content, environment: &Environment) -> impl View + 'a;
}
