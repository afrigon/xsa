use resource_loader::utils::CubeFace;
use xsa_core::terrain::{CubeTexel, HeightMapLevel, TexelSource};

use super::TexelRange;

// The narrowest cells, in texels of the level they bound.
const LEAF_TEXELS: u32 = 16;
// How far from a sample a cubic B-spline reads, in texels of the level it samples: the four texels around it, each
// covering half a texel to either side of its centre.
const SPLINE_REACH_TEXELS: u32 = 3;

// The lowest and highest height under every part of a height map level's faces: a min-max mip pyramid (maximum
// mipmaps, Tevs et al. 2008) whose cells, from `LEAF_TEXELS` wide up to a whole face, hold the range of the level's
// texels they cover. Every coarser level of the map averages these texels, so the pyramid bounds whatever any level
// shows at or above its own.
pub struct HeightBounds {
    level: u32,
    extent: u32,
    // From the leaves to one cell per face; each level face by face, its cells row by row.
    levels: Vec<Vec<TexelRange>>,
}

impl HeightBounds {
    pub fn new(heights: &HeightMapLevel) -> HeightBounds {
        let extent = heights.extent();
        let leaf = LEAF_TEXELS.min(extent);
        let leaves_per_side = extent / leaf;
        let mut leaves = Vec::with_capacity(CubeFace::ALL.len() * (leaves_per_side * leaves_per_side) as usize);

        for face in CubeFace::ALL {
            let texels = heights.face(face);

            for cell in 0..leaves_per_side * leaves_per_side {
                let (column, row) = (cell % leaves_per_side * leaf, cell / leaves_per_side * leaf);
                let mut range = TexelRange::EMPTY;

                for y in row..row + leaf {
                    let start = (y * extent + column) as usize;

                    for &value in &texels[start..start + leaf as usize] {
                        range = range.including(value);
                    }
                }

                leaves.push(range);
            }
        }

        let mut levels = vec![leaves];

        while let Some(finer) = levels.last()
            && finer.len() > CubeFace::ALL.len()
        {
            levels.push(HeightBounds::coarser(finer));
        }

        HeightBounds {
            level: heights.level(),
            extent,
            levels,
        }
    }

    // The bound of every height a tile of `face` at `depth`, column `x` and row `y` can show while its vertices sample
    // levels no coarser than `coarsest_level` of the map. It reads the cells over the tile widened by the B-spline's
    // reach at that level, from the pyramid level whose cells are at least that wide, so the widened area reaches at
    // most one cell into the adjacent faces. A reach wider than a face covers more than the face and its neighbours:
    // the whole body bounds it.
    pub fn range(&self, face: CubeFace, depth: u32, x: u32, y: u32, coarsest_level: u32) -> TexelRange {
        let reach = f64::from(SPLINE_REACH_TEXELS << coarsest_level.saturating_sub(self.level));
        let tile_size = f64::from(self.extent) / f64::from(depth).exp2();

        if reach > f64::from(self.extent) {
            return self.overall();
        }
        let leaf = LEAF_TEXELS.min(self.extent);
        let mut pyramid_level = 0;

        while pyramid_level + 1 < self.levels.len() && f64::from(leaf << pyramid_level) < tile_size.max(reach) {
            pyramid_level += 1;
        }

        let cells_per_side = (self.extent / leaf) >> pyramid_level;
        let cell_size = f64::from(leaf << pyramid_level);
        let first = |start: f64| ((start - reach) / cell_size).floor() as i32;
        let last = |start: f64| ((start + tile_size + reach) / cell_size).ceil() as i32 - 1;
        let (left, top) = (f64::from(x) * tile_size, f64::from(y) * tile_size);
        let mut range = TexelRange::EMPTY;

        for row in first(top)..=last(top) {
            for column in first(left)..=last(left) {
                let cell = |texel| self.cell(pyramid_level, cells_per_side, texel);

                range = match (CubeTexel { face, column, row }).source(cells_per_side as i32) {
                    TexelSource::Stored(texel) => range.union(cell(texel)),
                    TexelSource::Corner(texels) => {
                        texels.into_iter().fold(range, |range, texel| range.union(cell(texel)))
                    }
                };
            }
        }

        range
    }

    // Every height of the map.
    pub fn overall(&self) -> TexelRange {
        self.levels
            .last()
            .expect("a pyramid holds at least one level")
            .iter()
            .fold(TexelRange::EMPTY, |range, &cell| range.union(cell))
    }

    // The bytes it holds.
    pub fn size(&self) -> usize {
        self.levels.iter().map(Vec::len).sum::<usize>() * size_of::<TexelRange>()
    }

    fn cell(&self, pyramid_level: usize, cells_per_side: u32, texel: CubeTexel) -> TexelRange {
        let index =
            (texel.face.index() as u32 * cells_per_side + texel.row as u32) * cells_per_side + texel.column as u32;

        self.levels[pyramid_level][index as usize]
    }

    // Each cell the union of the four finer ones it covers.
    fn coarser(finer: &[TexelRange]) -> Vec<TexelRange> {
        let finer_side = ((finer.len() / CubeFace::ALL.len()) as f64).sqrt() as usize;
        let side = finer_side / 2;
        let mut cells = Vec::with_capacity(CubeFace::ALL.len() * side * side);

        for face in 0..CubeFace::ALL.len() {
            let face_cells = &finer[face * finer_side * finer_side..][..finer_side * finer_side];

            for cell in 0..side * side {
                let (column, row) = (cell % side * 2, cell / side * 2);
                let range = [(0, 0), (1, 0), (0, 1), (1, 1)]
                    .iter()
                    .fold(TexelRange::EMPTY, |range, &(dx, dy)| {
                        range.union(face_cells[(row + dy) * finer_side + column + dx])
                    });
                cells.push(range);
            }
        }

        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXTENT: u32 = 64;
    const RAISED: u16 = 50_000;
    const GROUND: u16 = 1_000;

    // Flat ground with one raised texel on `face`.
    fn bounds_with_peak(face: CubeFace, column: u32, row: u32) -> HeightBounds {
        let texels = CubeFace::ALL
            .iter()
            .flat_map(|&each| {
                (0..EXTENT * EXTENT).map(move |index| {
                    let peak = each == face && index == row * EXTENT + column;

                    if peak { RAISED } else { GROUND }
                })
            })
            .collect();
        let level = HeightMapLevel::new(0, EXTENT, texels, -1000.0, 1000.0).unwrap();

        HeightBounds::new(&level)
    }

    #[test]
    fn a_tile_over_a_peak_bounds_it_and_a_distant_one_does_not() {
        let bounds = bounds_with_peak(CubeFace::PositiveZ, 40, 40);

        assert_eq!(bounds.range(CubeFace::PositiveZ, 2, 2, 2, 0).maximum, RAISED);
        assert_eq!(bounds.range(CubeFace::PositiveZ, 2, 0, 0, 0).maximum, GROUND);
        assert_eq!(bounds.range(CubeFace::NegativeZ, 0, 0, 0, 0).maximum, GROUND);
    }

    // +Z's last column meets +X's first, both faces' rows running along −Y: the texel on that edge reaches the tile
    // beside it on +X, and no tile further along.
    #[test]
    fn a_peak_on_a_face_edge_bounds_the_adjacent_face() {
        let bounds = bounds_with_peak(CubeFace::PositiveZ, EXTENT - 1, EXTENT / 2);

        assert_eq!(bounds.range(CubeFace::PositiveX, 2, 0, 2, 0).maximum, RAISED);
        assert_eq!(bounds.range(CubeFace::PositiveX, 2, 3, 2, 0).maximum, GROUND);
    }

    #[test]
    fn coarser_samples_widen_the_bound() {
        let bounds = bounds_with_peak(CubeFace::PositiveZ, 40, 40);
        let neighbour = bounds.range(CubeFace::PositiveZ, 3, 1, 1, 0).maximum;
        let coarse = bounds.range(CubeFace::PositiveZ, 3, 1, 1, 4).maximum;

        assert_eq!(neighbour, GROUND);
        assert_eq!(coarse, RAISED);
    }

    // A reach wider than a face covers more than the face and its neighbours.
    #[test]
    fn a_reach_wider_than_a_face_is_bounded_by_the_whole_body() {
        let texels = (0..CubeFace::ALL.len() * 16)
            .map(|index| if index == 70 { RAISED } else { GROUND })
            .collect();
        let level = HeightMapLevel::new(0, 4, texels, -1000.0, 1000.0).unwrap();
        let bounds = HeightBounds::new(&level);

        assert_eq!(bounds.range(CubeFace::NegativeZ, 3, 1, 6, 2), bounds.overall());
        assert_eq!(bounds.range(CubeFace::NegativeZ, 0, 0, 0, 0).maximum, GROUND);
    }

    #[test]
    fn the_overall_range_spans_every_texel() {
        let bounds = bounds_with_peak(CubeFace::NegativeY, 3, 7);

        assert_eq!(
            bounds.overall(),
            TexelRange {
                minimum: GROUND,
                maximum: RAISED
            }
        );
    }
}
