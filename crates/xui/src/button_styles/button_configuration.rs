// The interaction state a button style draws, like SwiftUI's `ButtonStyle.Configuration`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ButtonConfiguration {
    pub is_hovered: bool,
    pub is_pressed: bool,
}
