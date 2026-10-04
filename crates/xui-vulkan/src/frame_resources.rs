use ash::vk;

use crate::host_buffer::HostBuffer;
use crate::memory_types::MemoryTypes;
use crate::primitive_instance::PrimitiveInstance;

const INITIAL_INSTANCE_CAPACITY: usize = 1024;
const INITIAL_STAGING_SIZE: usize = 64 * 1024;
const ATLAS_BINDING: u32 = 0;
const INSTANCES_BINDING: u32 = 1;

// The buffers and descriptor set one frame slot writes; only touched once the GPU is done with that slot.
pub(crate) struct FrameResources {
    instances: HostBuffer,
    staging: HostBuffer,
    descriptor_set: vk::DescriptorSet,
}

impl FrameResources {
    pub unsafe fn new(
        device: &ash::Device,
        memory_types: &MemoryTypes,
        descriptor_set: vk::DescriptorSet,
        atlas: vk::ImageView,
        sampler: vk::Sampler,
    ) -> anyhow::Result<FrameResources> {
        let mut instances = unsafe {
            HostBuffer::new(
                device,
                memory_types,
                INITIAL_INSTANCE_CAPACITY * size_of::<PrimitiveInstance>(),
                vk::BufferUsageFlags::STORAGE_BUFFER,
            )
        }?;
        let staging = match unsafe {
            HostBuffer::new(
                device,
                memory_types,
                INITIAL_STAGING_SIZE,
                vk::BufferUsageFlags::TRANSFER_SRC,
            )
        } {
            Ok(staging) => staging,
            Err(error) => {
                unsafe { instances.destroy(device) };
                return Err(error);
            }
        };
        let image_info = [vk::DescriptorImageInfo::default()
            .sampler(sampler)
            .image_view(atlas)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(ATLAS_BINDING)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info);
        unsafe { device.update_descriptor_sets(&[write], &[]) };
        let resources = FrameResources {
            instances,
            staging,
            descriptor_set,
        };
        resources.bind_instances(device);

        Ok(resources)
    }

    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }

    pub fn staging(&self) -> &HostBuffer {
        &self.staging
    }

    pub unsafe fn write_staging(
        &mut self,
        device: &ash::Device,
        memory_types: &MemoryTypes,
        bytes: &[u8],
    ) -> anyhow::Result<()> {
        if bytes.len() > self.staging.size() {
            let staging = unsafe {
                HostBuffer::new(
                    device,
                    memory_types,
                    bytes.len().next_power_of_two(),
                    vk::BufferUsageFlags::TRANSFER_SRC,
                )
            }?;
            unsafe { self.staging.destroy(device) };
            self.staging = staging;
        }

        self.staging.write(0, bytes)
    }

    pub unsafe fn write_instances(
        &mut self,
        device: &ash::Device,
        memory_types: &MemoryTypes,
        instances: &[PrimitiveInstance],
    ) -> anyhow::Result<()> {
        let bytes = PrimitiveInstance::bytes(instances);

        if bytes.len() > self.instances.size() {
            let buffer = unsafe {
                HostBuffer::new(
                    device,
                    memory_types,
                    bytes.len().next_power_of_two(),
                    vk::BufferUsageFlags::STORAGE_BUFFER,
                )
            }?;
            unsafe { self.instances.destroy(device) };
            self.instances = buffer;
            self.bind_instances(device);
        }

        self.instances.write(0, bytes)
    }

    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            self.instances.destroy(device);
            self.staging.destroy(device);
        }
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
