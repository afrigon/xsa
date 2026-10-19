use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::ensure;
use ash::vk;
use memory_vulkan::{BindlessTable, TexelBlock, TransferPriority};
use resource_loader::terrain::{HeightRange, PAGE_EXTENT, PageAddress, TerrainHeader, TerrainReader};
use resource_loader::utils::CubeFace;
use resource_vulkan::{LoadContext, MemoryUsage, PageFormat, PendingResource, ResourceKind};

// A terrain layer's `.terrain` file as far as streaming needs it before any page: the format and levels its header
// declares and, for heights, the ranges of its six level 0 pages, which together span every height of the body. Its
// pages stream through a streamed set once it is loaded.
pub struct TerrainLayerFile {
    header: TerrainHeader,
    level_zero_ranges: Vec<(PageAddress, HeightRange)>,
}

impl TerrainLayerFile {
    pub fn finest_level(&self) -> u32 {
        self.header.level_count - 1
    }

    // Empty unless its pages are heights.
    pub fn level_zero_ranges(&self) -> &[(PageAddress, HeightRange)] {
        &self.level_zero_ranges
    }

    pub fn page_format(&self) -> PageFormat {
        let format = self.header.format;
        let block_extent = format.block_extent();

        PageFormat {
            format: vk::Format::from_raw(format.vulkan_format().value as i32),
            extent: vk::Extent2D {
                width: PAGE_EXTENT,
                height: PAGE_EXTENT,
            },
            texel_block: TexelBlock {
                extent: vk::Extent2D {
                    width: block_extent,
                    height: block_extent,
                },
                size: format.block_size() as u32,
            },
        }
    }
}

impl ResourceKind for TerrainLayerFile {
    type Options = ();

    fn load(_: &LoadContext, path: &Path, _: &()) -> anyhow::Result<Box<dyn PendingResource<TerrainLayerFile>>> {
        let mut reader = TerrainReader::new(BufReader::new(File::open(path)?))?;
        let header = *reader.header();
        ensure!(header.level_count > 0, "the file has no levels");
        let mut level_zero_ranges = Vec::new();

        for face in CubeFace::ALL {
            for entry in reader.entries(face, 0)? {
                if let Some(range) = entry.range {
                    let address = PageAddress {
                        face,
                        level: 0,
                        key: entry.key,
                    };
                    level_zero_ranges.push((address, range));
                }
            }
        }

        Ok(Box::new(ReadLayerFile {
            file: Some(TerrainLayerFile {
                header,
                level_zero_ranges,
            }),
        }))
    }

    fn memory(&self) -> MemoryUsage {
        MemoryUsage::default()
    }
}

struct ReadLayerFile {
    // Taken by the first poll.
    file: Option<TerrainLayerFile>,
}

impl PendingResource<TerrainLayerFile> for ReadLayerFile {
    fn poll(&mut self, _: &BindlessTable) -> anyhow::Result<Option<TerrainLayerFile>> {
        Ok(self.file.take())
    }

    fn set_priority(&self, _: TransferPriority) {}
}
