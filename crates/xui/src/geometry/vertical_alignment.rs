#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

impl VerticalAlignment {
    pub fn offset(self, available: f32, used: f32) -> f32 {
        match self {
            VerticalAlignment::Top => 0.0,
            VerticalAlignment::Center => (available - used) / 2.0,
            VerticalAlignment::Bottom => available - used,
        }
    }
}
