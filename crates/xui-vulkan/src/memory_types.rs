use anyhow::Context;
use ash::vk;

use crate::vulkan_functions::VulkanFunctions;

pub(crate) struct MemoryTypes {
    properties: vk::PhysicalDeviceMemoryProperties,
}

impl MemoryTypes {
    pub unsafe fn new(functions: &VulkanFunctions) -> MemoryTypes {
        MemoryTypes {
            properties: unsafe {
                functions
                    .instance
                    .get_physical_device_memory_properties(functions.physical_device)
            },
        }
    }

    pub unsafe fn allocate(
        &self,
        device: &ash::Device,
        requirements: vk::MemoryRequirements,
        flags: vk::MemoryPropertyFlags,
    ) -> anyhow::Result<vk::DeviceMemory> {
        let memory_type = (0..self.properties.memory_type_count)
            .find(|&index| {
                requirements.memory_type_bits & (1 << index) != 0
                    && self.properties.memory_types[index as usize]
                        .property_flags
                        .contains(flags)
            })
            .with_context(|| format!("the GPU has no {flags:?} memory for this resource"))?;
        let info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type);

        Ok(unsafe { device.allocate_memory(&info, None) }?)
    }
}
