use resource_vulkan::ResourceHandle;

use super::gpu_data::TerrainData;
use super::terrain_buffer::TerrainBuffer;
use super::terrain_height::TerrainHeight;
use super::{GpuContext, TerrainHandle};
use crate::resource_kinds::TerrainHeightMap;

// What a terrain's loaded height map adds to a smooth sphere: its heights on the CPU and the data its shader reads.
// A terrain is drawn smooth until its buffer is uploaded.
pub(in crate::renderer) struct TerrainDisplacement {
    pub height: TerrainHeight,
    pub buffer: TerrainBuffer,
}

impl TerrainDisplacement {
    // A terrain's displacement in `displacements`, by terrain, once its buffer is uploaded; until then the terrain is
    // selected and drawn smooth.
    pub fn uploaded(
        displacements: &[Option<TerrainDisplacement>],
        terrain: TerrainHandle,
    ) -> Option<&TerrainDisplacement> {
        displacements
            .get(terrain.index)?
            .as_ref()
            .filter(|displacement| displacement.buffer.device_address().is_some())
    }

    // None until the height map is ready.
    pub fn of(
        gpu: &GpuContext,
        name: &str,
        height_map: &ResourceHandle<TerrainHeightMap>,
        radius: f64,
    ) -> anyhow::Result<Option<TerrainDisplacement>> {
        let Some(loaded) = height_map.get() else {
            return Ok(None);
        };
        let height = TerrainHeight::new(loaded.map().clone());
        let map = &height.map;
        let data = TerrainData {
            height_texture: loaded.texture().index(),
            minimum_height: (map.minimum() / radius) as f32,
            height_range: ((map.maximum() - map.minimum()) / radius) as f32,
            padding: 0,
        };

        Ok(Some(TerrainDisplacement {
            buffer: TerrainBuffer::new(gpu, name, data)?,
            height,
        }))
    }
}
