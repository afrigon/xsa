use super::{GpuData, TileHeightPage, TileLayerPage};

// Heights are read at four levels ending one level coarser than the tile's depth: vertices pick theirs by distance.
pub(in crate::renderer) const HEIGHT_LEVELS: usize = 4;
// Shading layers are read at the tile's depth and the two levels below it: one page, four, then sixteen, row by row.
pub(in crate::renderer) const LAYER_LEVELS: usize = 3;
pub(in crate::renderer) const LAYER_PAGES: usize = 1 + 4 + 16;
pub(in crate::renderer) const LAYERS: usize = 4;

// The pages one tile reads, at the same index as its tile data. Mirrored by `TilePages` in `geometry/tile.slang`.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TilePages {
    pub heights: [TileHeightPage; HEIGHT_LEVELS],
    pub layers: [[TileLayerPage; LAYER_PAGES]; LAYERS],
}

impl TilePages {
    pub const NONE: TilePages = TilePages {
        heights: [TileHeightPage::NONE; HEIGHT_LEVELS],
        layers: [[TileLayerPage::NONE; LAYER_PAGES]; LAYERS],
    };
}

const _: () = assert!(size_of::<TilePages>() == 736);

unsafe impl GpuData for TilePages {}
