mod bloom_push_constants;
mod frame_data;
mod ghost_data;
mod histogram_push_constants;
mod lens_push_constants;
mod object_data;
mod projected_light_data;
mod push_constants;
mod star_data;
mod taa_push_constants;
mod tile_data;
mod tile_height_page;
mod tile_layer_page;
mod tile_pages;
mod tonemap_push_constants;

pub(super) use bloom_push_constants::BloomPushConstants;
pub(super) use frame_data::FrameData;
pub(super) use ghost_data::GhostData;
pub(super) use histogram_push_constants::HistogramPushConstants;
pub(super) use lens_push_constants::LensPushConstants;
pub(super) use object_data::ObjectData;
pub(super) use projected_light_data::ProjectedLightData;
pub(super) use push_constants::PushConstants;
pub(super) use star_data::StarData;
pub(super) use taa_push_constants::TaaPushConstants;
pub(super) use tile_data::TileData;
pub(super) use tile_height_page::TileHeightPage;
pub(super) use tile_layer_page::TileLayerPage;
pub(super) use tile_pages::{HEIGHT_LEVELS, LAYER_LEVELS, LAYER_PAGES, TilePages};
pub(super) use tonemap_push_constants::TonemapPushConstants;

/// # Safety
///
/// Implementors are `#[repr(C)]` with no implicit padding, so every byte is initialized.
pub unsafe trait GpuData: Copy {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((self as *const Self).cast::<u8>(), size_of::<Self>()) }
    }
}
