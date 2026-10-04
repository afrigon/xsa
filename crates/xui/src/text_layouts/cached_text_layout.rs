use std::collections::HashMap;

use parley::{AlignmentOptions, Layout};

use crate::Size;

// A shaped text, broken into lines for one width at a time; sizes measured at other widths are remembered.
// Widths and sizes are in physical pixels.
pub(crate) struct CachedTextLayout {
    layout: Layout<()>,
    broken_at: Option<Option<u32>>,
    sizes: HashMap<Option<u32>, Size>,
    pub last_used: u64,
}

impl CachedTextLayout {
    pub fn new(layout: Layout<()>, frame: u64) -> CachedTextLayout {
        CachedTextLayout {
            layout,
            broken_at: None,
            sizes: HashMap::new(),
            last_used: frame,
        }
    }

    pub fn size(&mut self, max_width: Option<f32>) -> Size {
        let width_bits = max_width.map(f32::to_bits);

        if let Some(size) = self.sizes.get(&width_bits) {
            return *size;
        }

        let size = {
            let layout = self.broken_at(max_width);
            Size {
                width: layout.width(),
                height: layout.height(),
            }
        };
        self.sizes.insert(width_bits, size);

        size
    }

    pub fn broken_at(&mut self, max_width: Option<f32>) -> &Layout<()> {
        let width_bits = max_width.map(f32::to_bits);

        if self.broken_at != Some(width_bits) {
            self.layout.break_all_lines(max_width);
            self.layout.align(parley::Alignment::Start, AlignmentOptions::default());
            self.broken_at = Some(width_bits);
        }

        &self.layout
    }
}
