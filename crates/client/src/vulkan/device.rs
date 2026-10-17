mod suitable_device;

pub use suitable_device::SuitableDevice;

use anyhow::Context;
use ash::vk;
use ash::{ext, khr};
use memory_vulkan::MemoryContext;

use super::Instance;

pub struct Device {
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    queue_family: u32,
    queue: vk::Queue,
    max_sampler_anisotropy: f32,
    // Nanoseconds per timestamp tick; `None` when the queue cannot write timestamps.
    timestamp_period: Option<f32>,
    extended_dynamic_state3: Option<ext::extended_dynamic_state3::Device>,
}

impl Device {
    // The device is owned by the memory context, which destroys it once every object created from it is gone.
    pub fn create_memory(instance: &Instance, suitable: &SuitableDevice) -> anyhow::Result<MemoryContext> {
        let SuitableDevice {
            physical_device,
            queue_family,
            dynamic_polygon_mode,
            ..
        } = *suitable;
        let queue_priorities = [1.0];
        let queue_create_infos = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family)
            .queue_priorities(&queue_priorities)];
        let mut extensions = vec![
            khr::swapchain::NAME.as_ptr(),
            ext::swapchain_maintenance1::NAME.as_ptr(),
        ];

        if dynamic_polygon_mode {
            extensions.push(ext::extended_dynamic_state3::NAME.as_ptr());
        }

        let features = vk::PhysicalDeviceFeatures::default()
            .fill_mode_non_solid(true)
            .geometry_shader(true)
            .sampler_anisotropy(true)
            .shader_int64(true)
            .shader_storage_image_extended_formats(true);
        let mut vulkan_11_features = vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true);
        let mut vulkan_12_features = vk::PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .timeline_semaphore(true)
            .runtime_descriptor_array(true)
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_storage_image_update_after_bind(true)
            .shader_sampled_image_array_non_uniform_indexing(true);
        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);
        let mut swapchain_maintenance1_features =
            vk::PhysicalDeviceSwapchainMaintenance1FeaturesEXT::default().swapchain_maintenance1(true);
        let mut extended_dynamic_state3_features =
            vk::PhysicalDeviceExtendedDynamicState3FeaturesEXT::default().extended_dynamic_state3_polygon_mode(true);
        let mut create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extensions)
            .enabled_features(&features)
            .push_next(&mut vulkan_11_features)
            .push_next(&mut vulkan_12_features)
            .push_next(&mut vulkan_13_features)
            .push_next(&mut swapchain_maintenance1_features);

        if dynamic_polygon_mode {
            create_info = create_info.push_next(&mut extended_dynamic_state3_features);
        }

        MemoryContext::new(instance.handle(), physical_device, &create_info).context("creating the Vulkan device")
    }

    pub fn new(instance: &Instance, memory: &MemoryContext, suitable: &SuitableDevice) -> Self {
        let SuitableDevice {
            physical_device,
            queue_family,
            dynamic_polygon_mode,
            ..
        } = *suitable;
        let device = memory.device().clone();
        let queue = unsafe { device.get_device_queue(queue_family, 0) };
        let extended_dynamic_state3 =
            dynamic_polygon_mode.then(|| ext::extended_dynamic_state3::Device::new(instance.handle(), &device));
        let limits = unsafe { instance.handle().get_physical_device_properties(physical_device) }.limits;
        let queue_families = unsafe {
            instance
                .handle()
                .get_physical_device_queue_family_properties(physical_device)
        };
        let timestamp_period =
            (queue_families[queue_family as usize].timestamp_valid_bits > 0).then_some(limits.timestamp_period);

        Self {
            physical_device,
            device,
            queue_family,
            queue,
            max_sampler_anisotropy: limits.max_sampler_anisotropy,
            timestamp_period,
            extended_dynamic_state3,
        }
    }

    pub fn handle(&self) -> &ash::Device {
        &self.device
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn queue_family(&self) -> u32 {
        self.queue_family
    }

    pub fn queue(&self) -> vk::Queue {
        self.queue
    }

    pub fn max_sampler_anisotropy(&self) -> f32 {
        self.max_sampler_anisotropy
    }

    pub fn timestamp_period(&self) -> Option<f32> {
        self.timestamp_period
    }

    pub fn extended_dynamic_state3(&self) -> Option<&ext::extended_dynamic_state3::Device> {
        self.extended_dynamic_state3.as_ref()
    }
}
