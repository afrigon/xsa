use ash::vk;

use crate::VideoError;
use crate::encoder_capabilities::EncoderCapabilities;

const MIN_CODING_BLOCK_LOG2: u32 = 3;

// The frames' size, the size they are coded at (rounded up to what the encoder works in; the stream crops the padding
// away) and the block sizes coding uses.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PictureGeometry {
    pub extent: vk::Extent2D,
    pub coded_extent: vk::Extent2D,
    pub ctb_log2: u32,
    pub min_transform_log2: u32,
    pub max_transform_log2: u32,
}

impl PictureGeometry {
    // 4:2:0 stores one chroma sample per 2×2 pixels, so a frame's size is even.
    pub fn new(extent: vk::Extent2D, capabilities: &EncoderCapabilities) -> Result<PictureGeometry, VideoError> {
        if !extent.width.is_multiple_of(2)
            || !extent.height.is_multiple_of(2)
            || extent.width == 0
            || extent.height == 0
        {
            return Err(VideoError::Unsupported(format!(
                "a {}×{} frame: 4:2:0 frames have an even size",
                extent.width, extent.height
            )));
        }

        let ctb_log2 = [
            (vk::VideoEncodeH265CtbSizeFlagsKHR::TYPE_64, 6),
            (vk::VideoEncodeH265CtbSizeFlagsKHR::TYPE_32, 5),
            (vk::VideoEncodeH265CtbSizeFlagsKHR::TYPE_16, 4),
        ]
        .into_iter()
        .find(|(flag, _)| capabilities.ctb_sizes.contains(*flag))
        .map(|(_, log2)| log2)
        .ok_or_else(|| VideoError::Unsupported("any coding tree block size".to_string()))?;
        let transform_sizes = [
            (vk::VideoEncodeH265TransformBlockSizeFlagsKHR::TYPE_4, 2),
            (vk::VideoEncodeH265TransformBlockSizeFlagsKHR::TYPE_8, 3),
            (vk::VideoEncodeH265TransformBlockSizeFlagsKHR::TYPE_16, 4),
            (vk::VideoEncodeH265TransformBlockSizeFlagsKHR::TYPE_32, 5),
        ];
        let supported_transforms = transform_sizes
            .iter()
            .filter(|(flag, log2)| capabilities.transform_block_sizes.contains(*flag) && *log2 <= ctb_log2);
        // The smallest transform block must be smaller than the smallest coding block.
        let min_transform_log2 = supported_transforms
            .clone()
            .map(|(_, log2)| *log2)
            .find(|log2| *log2 < MIN_CODING_BLOCK_LOG2)
            .ok_or_else(|| VideoError::Unsupported("4×4 transform blocks".to_string()))?;
        let max_transform_log2 = supported_transforms
            .map(|(_, log2)| *log2)
            .max()
            .unwrap_or(min_transform_log2);
        let alignment = capabilities
            .picture_granularity
            .width
            .max(capabilities.picture_granularity.height)
            .max(1 << ctb_log2);
        let coded_extent = vk::Extent2D {
            width: extent.width.next_multiple_of(alignment),
            height: extent.height.next_multiple_of(alignment),
        };

        if coded_extent.width > capabilities.max_coded_extent.width
            || coded_extent.height > capabilities.max_coded_extent.height
        {
            return Err(VideoError::Unsupported(format!(
                "a {}×{} frame",
                extent.width, extent.height
            )));
        }

        Ok(PictureGeometry {
            extent,
            coded_extent,
            ctb_log2,
            min_transform_log2,
            max_transform_log2,
        })
    }

    pub fn min_coding_block_log2(&self) -> u32 {
        MIN_CODING_BLOCK_LOG2
    }
}
