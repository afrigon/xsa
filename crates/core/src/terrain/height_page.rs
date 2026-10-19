use std::sync::Arc;

use resource_loader::terrain::{HeightRange, PAGE_BORDER, PAGE_EXTENT};

// One page of a terrain's heights as stored: 16-bit values row by row, border included, spanning its own range.
#[derive(Clone, Debug)]
pub struct HeightPage {
    pub range: HeightRange,
    pub values: Arc<[u8]>,
}

impl HeightPage {
    // Meters relative to the body's radius at a texel of the page's own area; the border lies at -BORDER and beyond.
    pub fn height(&self, x: i32, y: i32) -> f64 {
        let column = (x + PAGE_BORDER as i32) as usize;
        let row = (y + PAGE_BORDER as i32) as usize;
        let index = (row * PAGE_EXTENT as usize + column) * size_of::<u16>();
        let value = u16::from_le_bytes([self.values[index], self.values[index + 1]]);

        f64::from(self.range.height(value))
    }

    // The weights of the four texels around a cubic B-spline sample at fraction `f` past the second.
    pub fn spline_weights(f: f64) -> [f64; 4] {
        let f2 = f * f;
        let f3 = f2 * f;

        [
            (1.0 - 3.0 * f + 3.0 * f2 - f3) / 6.0,
            (4.0 - 6.0 * f2 + 3.0 * f3) / 6.0,
            (1.0 + 3.0 * f + 3.0 * f2 - 3.0 * f3) / 6.0,
            f3 / 6.0,
        ]
    }
}
