use std::fmt;

use resource_loader::terrain::{CubeFace, PageKey};

/// Which pages of one face at one level are present, scaled to at most `MAP_WIDTH` characters across.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CoverageMap {
    pub face: CubeFace,
    pub level: u32,
    // The width and height in pages of the square each character covers.
    cell_extent: u64,
    width: u64,
    counts: Vec<u64>,
}

impl CoverageMap {
    pub const MAP_WIDTH: u64 = 64;

    /// The map of a face's present pages, given by keys inside the level.
    pub fn new(face: CubeFace, level: u32, keys: impl IntoIterator<Item = PageKey>) -> CoverageMap {
        let pages_per_side = 1u64 << level;
        let cell_extent = pages_per_side.div_ceil(Self::MAP_WIDTH);
        let width = pages_per_side / cell_extent;
        let mut counts = vec![0; (width * width) as usize];

        for key in keys {
            let column = u64::from(key.x()) / cell_extent;
            let row = u64::from(key.y()) / cell_extent;
            counts[(row * width + column) as usize] += 1;
        }

        CoverageMap {
            face,
            level,
            cell_extent,
            width,
            counts,
        }
    }
}

impl fmt::Display for CoverageMap {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let full = self.cell_extent * self.cell_extent;
        writeln!(
            formatter,
            "{} at level {}, a character per {}×{} pages: # all present, + some, . none",
            self.face, self.level, self.cell_extent, self.cell_extent
        )?;

        for row in self.counts.chunks(self.width as usize) {
            let line: String = row
                .iter()
                .map(|&count| match count {
                    0 => '.',
                    count if count == full => '#',
                    _ => '+',
                })
                .collect();
            writeln!(formatter, "  {line}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_coarse_level_maps_a_character_per_page() {
        let keys = [PageKey::from_coordinates(0, 0), PageKey::from_coordinates(1, 1)];
        let map = CoverageMap::new(CubeFace::PositiveZ, 1, keys);

        assert_eq!(
            map.to_string(),
            "+z at level 1, a character per 1×1 pages: # all present, + some, . none\n  #.\n  .#\n"
        );
    }

    #[test]
    fn a_fine_level_scales_down_to_the_map_width() {
        let keys = (0..256).flat_map(|y| (0..128).map(move |x| PageKey::from_coordinates(x, y)));
        let map = CoverageMap::new(
            CubeFace::NegativeX,
            8,
            keys.chain([PageKey::from_coordinates(255, 255)]),
        );
        let rows: Vec<String> = map.to_string().lines().skip(1).map(str::to_owned).collect();

        assert_eq!(rows.len(), 64);
        assert_eq!(rows[0], format!("  {}{}", "#".repeat(32), ".".repeat(32)));
        assert_eq!(rows[63], format!("  {}{}+", "#".repeat(32), ".".repeat(31)));
    }
}
