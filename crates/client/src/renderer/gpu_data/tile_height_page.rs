use super::GpuData;
use super::tile_layer_page::TileLayerPage;

// Where a tile reads its heights at one level, with the range the page's 16-bit values span, in radii relative to the
// radius.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TileHeightPage {
    pub page: TileLayerPage,
    pub minimum: f32,
    pub span: f32,
}

impl TileHeightPage {
    pub const NONE: TileHeightPage = TileHeightPage {
        page: TileLayerPage::NONE,
        minimum: 0.0,
        span: 0.0,
    };
}

const _: () = assert!(size_of::<TileHeightPage>() == 16);

unsafe impl GpuData for TileHeightPage {}
