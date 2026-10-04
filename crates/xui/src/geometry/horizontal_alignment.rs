#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HorizontalAlignment {
    Leading,
    Center,
    Trailing,
}

impl HorizontalAlignment {
    pub fn offset(self, available: f32, used: f32) -> f32 {
        match self {
            HorizontalAlignment::Leading => 0.0,
            HorizontalAlignment::Center => (available - used) / 2.0,
            HorizontalAlignment::Trailing => available - used,
        }
    }
}
