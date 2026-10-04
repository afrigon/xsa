#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub leading: f32,
    pub bottom: f32,
    pub trailing: f32,
}

impl EdgeInsets {
    pub fn top(amount: f32) -> EdgeInsets {
        EdgeInsets {
            top: amount,
            ..EdgeInsets::default()
        }
    }

    pub fn all(amount: f32) -> EdgeInsets {
        EdgeInsets {
            top: amount,
            leading: amount,
            bottom: amount,
            trailing: amount,
        }
    }

    pub fn horizontal(self) -> f32 {
        self.leading + self.trailing
    }

    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}
