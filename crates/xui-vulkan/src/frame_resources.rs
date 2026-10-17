use ash::vk;
use memory_vulkan::{Buffer, MemoryContext, MemoryLocation};

use crate::primitive_instance::PrimitiveInstance;

const INITIAL_INSTANCE_CAPACITY: usize = 1024;
const INITIAL_STAGING_SIZE: usize = 64 * 1024;
const ATLAS_BINDING: u32 = 0;
const INSTANCES_BINDING: u32 = 1;

// The buffers and descriptor set one frame slot writes; only touched once the GPU is done with that slot. The CPU
// writes the buffers directly, in memory the GPU sees writes to at submission.
pub(crate) struct FrameResources {
    instances: Buffer,
    staging: Buffer,
    descriptor_set: vk::DescriptorSet,
}

impl FrameResources {
    pub fn new(
        memory: &MemoryContext,
        descriptor_set: vk::DescriptorSet,
        atlas: vk::ImageView,
        sampler: vk::Sampler,
    ) -> anyhow::Result<FrameResources> {
        let instances =
            FrameResources::create_instances(memory, INITIAL_INSTANCE_CAPACITY * size_of::<PrimitiveInstance>())?;
        let staging = FrameResources::create_staging(memory, INITIAL_STAGING_SIZE)?;
        let image_info = [vk::DescriptorImageInfo::default()
            .sampler(sampler)
            .image_view(atlas)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(ATLAS_BINDING)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info);
        unsafe { memory.device().update_descriptor_sets(&[write], &[]) };
        let resources = FrameResources {
            instances,
            staging,
            descriptor_set,
        };
        resources.bind_instances(memory.device());

        Ok(resources)
    }

    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }

    pub fn staging(&self) -> &Buffer {
        &self.staging
    }

    pub fn write_staging(&mut self, memory: &MemoryContext, bytes: &[u8]) -> anyhow::Result<()> {
        if bytes.len() as u64 > self.staging.size() {
            self.staging = FrameResources::create_staging(memory, bytes.len().next_power_of_two())?;
        }

        Ok(self.staging.write(bytes)?)
    }

    pub fn write_instances(&mut self, memory: &MemoryContext, instances: &[PrimitiveInstance]) -> anyhow::Result<()> {
        let bytes = PrimitiveInstance::bytes(instances);

        if bytes.len() as u64 > self.instances.size() {
            self.instances = FrameResources::create_instances(memory, bytes.len().next_power_of_two())?;
            self.bind_instances(memory.device());
        }

        Ok(self.instances.write(bytes)?)
    }

    fn create_instances(memory: &MemoryContext, size: usize) -> anyhow::Result<Buffer> {
        Ok(Buffer::new(
            memory,
            "interface instances",
            size as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            MemoryLocation::CpuToGpu,
        )?)
    }

    fn create_staging(memory: &MemoryContext, size: usize) -> anyhow::Result<Buffer> {
        Ok(Buffer::new(
            memory,
            "interface atlas staging",
            size as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?)
    }

    fn bind_instances(&self, device: &ash::Device) {
        let buffer_info = [vk::DescriptorBufferInfo::default()
            .buffer(self.instances.handle())
            .range(vk::WHOLE_SIZE)];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(self.descriptor_set)
            .dst_binding(INSTANCES_BINDING)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(&buffer_info);
        unsafe { device.update_descriptor_sets(&[write], &[]) };
    }
}
