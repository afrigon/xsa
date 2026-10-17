use ash::vk;

// One copy command of a batch.
pub(crate) enum TransferCommand {
    BufferToImage {
        source: vk::Buffer,
        image: vk::Image,
        region: vk::BufferImageCopy,
    },
    BufferToBuffer {
        source: vk::Buffer,
        destination: vk::Buffer,
        region: vk::BufferCopy,
    },
}

impl TransferCommand {
    pub unsafe fn record(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            match self {
                TransferCommand::BufferToImage { source, image, region } => device.cmd_copy_buffer_to_image(
                    command_buffer,
                    *source,
                    *image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    std::slice::from_ref(region),
                ),
                TransferCommand::BufferToBuffer {
                    source,
                    destination,
                    region,
                } => device.cmd_copy_buffer(command_buffer, *source, *destination, std::slice::from_ref(region)),
            }
        }
    }
}
