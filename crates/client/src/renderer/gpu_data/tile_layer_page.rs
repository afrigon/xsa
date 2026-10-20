use resource_loader::terrain::PageAddress;
use resource_vulkan::PagePlacement;

use super::GpuData;

// Where a tile reads one page of a layer: the bindless slot of its atlas chunk with the level of the page it holds,
// which is the page asked for or a coarser one standing in, and the page's texel offset in the chunk.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TileLayerPage {
    pub slot_and_level: u32,
    pub offset: u32,
}

impl TileLayerPage {
    // No page covering it is resident.
    pub const NONE: TileLayerPage = TileLayerPage {
        slot_and_level: u32::MAX,
        offset: 0,
    };
    const LEVEL_SHIFT: u32 = 27;
    const OFFSET_SHIFT: u32 = 16;

    pub fn new(placement: PagePlacement<PageAddress>) -> TileLayerPage {
        assert!(
            placement.slot < 1 << TileLayerPage::LEVEL_SHIFT,
            "bindless slot {} out of range",
            placement.slot
        );

        TileLayerPage {
            slot_and_level: placement.slot | placement.page.level << TileLayerPage::LEVEL_SHIFT,
            offset: placement.offset.x as u32 | (placement.offset.y as u32) << TileLayerPage::OFFSET_SHIFT,
        }
    }
}

const _: () = assert!(size_of::<TileLayerPage>() == 8);

unsafe impl GpuData for TileLayerPage {}
