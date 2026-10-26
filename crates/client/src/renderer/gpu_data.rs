mod biome_data;
mod bloom_push_constants;
mod copy_push_constants;
mod frame_data;
mod ghost_data;
mod histogram_push_constants;
mod lens_push_constants;
mod mesh_draw_data;
mod noise_anchor;
mod object_data;
mod projected_light_data;
mod push_constants;
mod scatter_draw_data;
mod scatter_draw_push_constants;
mod scatter_instance_data;
mod scatter_model_data;
mod scatter_page_data;
mod scatter_push_constants;
mod scatter_use_data;
mod star_data;
mod surface_anchor_data;
mod surface_data;
mod taa_push_constants;
mod terrain_data;
mod terrain_draw_data;
mod tile_data;
mod tonemap_push_constants;

pub(super) use biome_data::BiomeData;
pub(super) use bloom_push_constants::BloomPushConstants;
pub(super) use copy_push_constants::CopyPushConstants;
pub(super) use frame_data::FrameData;
pub(super) use ghost_data::GhostData;
pub(super) use histogram_push_constants::HistogramPushConstants;
pub(super) use lens_push_constants::LensPushConstants;
pub(super) use mesh_draw_data::MeshDrawData;
pub(super) use noise_anchor::NoiseAnchor;
pub(super) use object_data::ObjectData;
pub(super) use projected_light_data::ProjectedLightData;
pub(super) use push_constants::PushConstants;
pub(super) use scatter_draw_data::ScatterDrawData;
pub(super) use scatter_draw_push_constants::ScatterDrawPushConstants;
pub(super) use scatter_instance_data::ScatterInstanceData;
pub(super) use scatter_model_data::ScatterModelData;
pub(super) use scatter_page_data::ScatterPageData;
pub(super) use scatter_push_constants::ScatterPushConstants;
pub(super) use scatter_use_data::ScatterUseData;
pub(super) use star_data::StarData;
pub(super) use surface_anchor_data::SurfaceAnchorData;
pub(super) use surface_data::SurfaceData;
pub(super) use taa_push_constants::TaaPushConstants;
pub(super) use terrain_data::TerrainData;
pub(super) use terrain_draw_data::TerrainDrawData;
pub(super) use tile_data::TileData;
pub(super) use tonemap_push_constants::TonemapPushConstants;

/// # Safety
///
/// Implementors are `#[repr(C)]` with no implicit padding, so every byte is initialized.
pub unsafe trait GpuData: Copy {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((self as *const Self).cast::<u8>(), size_of::<Self>()) }
    }
}
