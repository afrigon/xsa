use ash::vk;
use memory_vulkan::{ReadbackTicket, TransferStatus};

use super::CapturedImage;

pub(super) const BYTES_PER_PIXEL: usize = 4;

// A rendered frame on its way back from the GPU.
pub(super) struct CaptureReadback {
    extent: vk::Extent2D,
    format: vk::Format,
    readback: ReadbackTicket,
}

impl CaptureReadback {
    pub(super) fn new(extent: vk::Extent2D, format: vk::Format, readback: ReadbackTicket) -> CaptureReadback {
        CaptureReadback {
            extent,
            format,
            readback,
        }
    }

    // The image once its pixels are back on the CPU.
    pub(super) fn poll(&mut self) -> anyhow::Result<Option<CapturedImage>> {
        let mut pixels = match self.readback.poll() {
            TransferStatus::Pending => return Ok(None),
            TransferStatus::Complete(pixels) => pixels,
            TransferStatus::Failed(err) => return Err(err.into()),
        };

        match self.format {
            vk::Format::B8G8R8A8_SRGB | vk::Format::B8G8R8A8_UNORM => {
                for pixel in pixels.as_chunks_mut::<BYTES_PER_PIXEL>().0 {
                    pixel.swap(0, 2);
                }
            }
            vk::Format::R8G8B8A8_SRGB | vk::Format::R8G8B8A8_UNORM => {}
            format => anyhow::bail!("capturing a {format:?} swapchain is not supported"),
        }

        Ok(Some(CapturedImage {
            width: self.extent.width,
            height: self.extent.height,
            rgba: pixels,
        }))
    }
}
