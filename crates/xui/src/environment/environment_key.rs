// A typed slot in the environment, like SwiftUI's `EnvironmentKey`: views read `K::Value`, and get
// `default_value` until an ancestor sets one.
pub trait EnvironmentKey: 'static {
    type Value: Clone + 'static;

    fn default_value() -> Self::Value;
}
