use std::path::Path;
use std::sync::Arc;

use anyhow::ensure;
use memory_vulkan::{BindlessTable, TransferPriority};
use resource_loader::texture::{ColorSpace, TextureFormat, TextureImage};
use resource_vulkan::{LoadContext, MemoryUsage, PendingResource, PendingTexture, ResourceKind, Texture};
use xsa_core::terrain::HeightMap;

use super::HeightMapRange;

// A body's heights: the texture the renderer displaces its terrain with, and the decoded top level the simulation
// reads the ground from.
pub struct TerrainHeightMap {
    texture: Texture,
    map: Arc<HeightMap>,
}

impl TerrainHeightMap {
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn map(&self) -> &Arc<HeightMap> {
        &self.map
    }
}

impl ResourceKind for TerrainHeightMap {
    type Options = HeightMapRange;

    fn load(
        context: &LoadContext,
        path: &Path,
        range: &HeightMapRange,
    ) -> anyhow::Result<Box<dyn PendingResource<TerrainHeightMap>>> {
        let image = TextureImage::read(path, ColorSpace::Linear)?;
        ensure!(
            image.format() == TextureFormat::R16Unorm,
            "a height map must be R16_UNORM, not {:?}",
            image.format()
        );

        let width = image.width() as usize;
        let height = image.height() as usize;
        let texels = image.layers()[0][..width * height * size_of::<u16>()]
            .as_chunks::<{ size_of::<u16>() }>()
            .0
            .iter()
            .map(|&bytes| u16::from_le_bytes(bytes))
            .collect();
        let map = HeightMap::new(width, height, texels, range.minimum, range.maximum)?;

        Ok(Box::new(PendingHeightMap {
            texture: Texture::upload(context, image)?,
            map: Some(Arc::new(map)),
        }))
    }

    fn memory(&self) -> MemoryUsage {
        self.texture.memory()
            + MemoryUsage {
                video: 0,
                ram: size_of_val(self.map.texels()) as u64,
            }
    }
}

struct PendingHeightMap {
    texture: PendingTexture,
    // Taken when the texture is ready.
    map: Option<Arc<HeightMap>>,
}

impl PendingResource<TerrainHeightMap> for PendingHeightMap {
    fn poll(&mut self, bindless: &BindlessTable) -> anyhow::Result<Option<TerrainHeightMap>> {
        Ok(self.texture.poll(bindless)?.map(|texture| TerrainHeightMap {
            texture,
            map: self.map.take().expect("a pending height map becomes ready once"),
        }))
    }

    fn set_priority(&self, priority: TransferPriority) {
        self.texture.set_priority(priority);
    }
}
