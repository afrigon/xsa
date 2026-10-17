use ash::{khr, vk};
use gpu_allocator::vulkan::{Allocation, Allocator};

use crate::MemoryError;

/// One piece of deferred work: an object to destroy, memory to free, or an action to run.
pub enum Deletion {
    Buffer(vk::Buffer),
    Image(vk::Image),
    ImageView(vk::ImageView),
    Sampler(vk::Sampler),
    Pipeline(vk::Pipeline),
    PipelineLayout(vk::PipelineLayout),
    DescriptorPool(vk::DescriptorPool),
    DescriptorSetLayout(vk::DescriptorSetLayout),
    ShaderModule(vk::ShaderModule),
    Semaphore(vk::Semaphore),
    CommandPool(vk::CommandPool),
    QueryPool(vk::QueryPool),
    Swapchain(vk::SwapchainKHR),
    Allocation(Allocation),
    Action(Box<dyn FnOnce(&ash::Device) + Send>),
}

impl Deletion {
    pub(crate) fn run(
        self,
        device: &ash::Device,
        swapchain: &khr::swapchain::Device,
        allocator: &mut Allocator,
    ) -> Result<(), MemoryError> {
        unsafe {
            match self {
                Deletion::Buffer(buffer) => device.destroy_buffer(buffer, None),
                Deletion::Image(image) => device.destroy_image(image, None),
                Deletion::ImageView(view) => device.destroy_image_view(view, None),
                Deletion::Sampler(sampler) => device.destroy_sampler(sampler, None),
                Deletion::Pipeline(pipeline) => device.destroy_pipeline(pipeline, None),
                Deletion::PipelineLayout(layout) => device.destroy_pipeline_layout(layout, None),
                Deletion::DescriptorPool(pool) => device.destroy_descriptor_pool(pool, None),
                Deletion::DescriptorSetLayout(layout) => device.destroy_descriptor_set_layout(layout, None),
                Deletion::ShaderModule(module) => device.destroy_shader_module(module, None),
                Deletion::Semaphore(semaphore) => device.destroy_semaphore(semaphore, None),
                Deletion::CommandPool(pool) => device.destroy_command_pool(pool, None),
                Deletion::QueryPool(pool) => device.destroy_query_pool(pool, None),
                Deletion::Swapchain(handle) => swapchain.destroy_swapchain(handle, None),
                Deletion::Allocation(allocation) => allocator.free(allocation)?,
                Deletion::Action(action) => action(device),
            }
        }

        Ok(())
    }
}
