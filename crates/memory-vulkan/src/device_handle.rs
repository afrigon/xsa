use ash::vk;
use gpu_allocator::vulkan::Allocation;

use crate::Deletion;

/// Something created from the device that the deletion queue can destroy or free.
pub trait DeviceHandle {
    fn into_deletion(self) -> Deletion;
}

macro_rules! device_handles {
    ($($handle:ty => $variant:ident),* $(,)?) => {
        $(
            impl DeviceHandle for $handle {
                fn into_deletion(self) -> Deletion {
                    Deletion::$variant(self)
                }
            }
        )*
    };
}

device_handles! {
    vk::Buffer => Buffer,
    vk::Image => Image,
    vk::ImageView => ImageView,
    vk::Sampler => Sampler,
    vk::Pipeline => Pipeline,
    vk::PipelineLayout => PipelineLayout,
    vk::DescriptorPool => DescriptorPool,
    vk::DescriptorSetLayout => DescriptorSetLayout,
    vk::ShaderModule => ShaderModule,
    vk::Semaphore => Semaphore,
    vk::CommandPool => CommandPool,
    vk::QueryPool => QueryPool,
    vk::SwapchainKHR => Swapchain,
    Allocation => Allocation,
}
