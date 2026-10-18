/// The heights a page's 16-bit values span, in metres relative to the body's radius.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct HeightRange {
    pub minimum: f32,
    pub maximum: f32,
}

impl HeightRange {
    const VALUE_MAXIMUM: f32 = u16::MAX as f32;

    /// The range spanning every height given, or `None` when there are none.
    pub fn spanning(heights: impl IntoIterator<Item = f32>) -> Option<HeightRange> {
        heights.into_iter().fold(None, |range, height| {
            Some(match range {
                None => HeightRange {
                    minimum: height,
                    maximum: height,
                },
                Some(range) => HeightRange {
                    minimum: range.minimum.min(height),
                    maximum: range.maximum.max(height),
                },
            })
        })
    }

    /// The height a stored value stands for.
    pub fn height(self, value: u16) -> f32 {
        self.minimum + f32::from(value) / Self::VALUE_MAXIMUM * (self.maximum - self.minimum)
    }

    /// The stored value nearest to a height within the range.
    pub fn value(self, height: f32) -> u16 {
        let span = self.maximum - self.minimum;

        if span == 0.0 {
            return 0;
        }

        ((height - self.minimum) / span * Self::VALUE_MAXIMUM).round() as u16
    }
}
