use ash::vk;
use memory_vulkan::TexelBlock;

/// The size and texel format of a set's pages, one mip level each. Sets with the same format share a pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PageFormat {
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub texel_block: TexelBlock,
}

impl PageFormat {
    /// The bytes of one page: rows of texel blocks, packed without padding.
    pub fn page_size(&self) -> usize {
        let block = self.texel_block;
        let columns = self.extent.width.div_ceil(block.extent.width) as usize;
        let rows = self.extent.height.div_ceil(block.extent.height) as usize;

        columns * rows * block.size as usize
    }
}
