use crate::Environment;

// What a view's body can read, like the property wrappers of a SwiftUI view.
pub struct Context {
    environment: Environment,
}

impl Context {
    pub(crate) fn new(environment: Environment) -> Context {
        Context { environment }
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }
}
