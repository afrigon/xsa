use super::GpuData;

// A terrain's maps as its tiles read them, one per terrain. Mirrored by `TerrainData` in `terrain/tile.slang`. Slots
// are NO_TEXTURE for a map the terrain lacks or has not loaded; heights are in radii relative to the radius, from their
// level 0, `height_extent` texels wide, through `height_levels` levels, of which those from `height_first_level` are
// resident. The colour layer's level 0 is `color_extent` texels wide.
#[repr(C)]
#[derive(Clone, Copy)]
pub(in crate::renderer) struct TerrainData {
    pub height_texture: u32,
    pub height_extent: u32,
    pub height_levels: u32,
    pub height_first_level: u32,
    pub height_minimum: f32,
    pub height_span: f32,
    pub layer_textures: [u32; 4],
    pub color_extent: u32,
    pub padding: u32,
}

const _: () = assert!(size_of::<TerrainData>() == 48);

unsafe impl GpuData for TerrainData {}
