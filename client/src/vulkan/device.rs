mod suitable_device;

use anyhow::Context;
use ash::vk;
use ash::{ext, khr};

use super::{Instance, Surface};
use suitable_device::SuitableDevice;

pub struct Device {
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    queue_family: u32,
    queue: vk::Queue,
    max_sampler_anisotropy: f32,
    extended_dynamic_state3: Option<ext::extended_dynamic_state3::Device>,
}

impl Device {
    pub fn new(instance: &Instance, surface: &Surface) -> anyhow::Result<Self> {
        let SuitableDevice {
            physical_device,
            queue_family,
            ..
        } = SuitableDevice::select(instance.handle(), surface)?;

        let queue_priorities = [1.0];
        let queue_create_infos = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family)
            .queue_priorities(&queue_priorities)];
        let supports_polygon_mode = Device::supports_dynamic_polygon_mode(instance.handle(), physical_device)?;
        let mut extensions = vec![khr::swapchain::NAME.as_ptr()];
        if supports_polygon_mode {
            extensions.push(ext::extended_dynamic_state3::NAME.as_ptr());
        }
        let features = vk::PhysicalDeviceFeatures::default()
            .fill_mode_non_solid(true)
            .geometry_shader(true)
            .sampler_anisotropy(true)
            .shader_storage_image_extended_formats(true);
        let mut vulkan_11_features = vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true);
        let mut vulkan_12_features = vk::PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .runtime_descriptor_array(true)
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
            .descriptor_binding_storage_image_update_after_bind(true)
            .shader_sampled_image_array_non_uniform_indexing(true);
        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);
        let mut extended_dynamic_state3_features =
            vk::PhysicalDeviceExtendedDynamicState3FeaturesEXT::default().extended_dynamic_state3_polygon_mode(true);
        let mut create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extensions)
            .enabled_features(&features)
            .push_next(&mut vulkan_11_features)
            .push_next(&mut vulkan_12_features)
            .push_next(&mut vulkan_13_features);
        if supports_polygon_mode {
            create_info = create_info.push_next(&mut extended_dynamic_state3_features);
        }
        let device = unsafe { instance.handle().create_device(physical_device, &create_info, None) }
            .context("creating the Vulkan device")?;
        let queue = unsafe { device.get_device_queue(queue_family, 0) };
        let extended_dynamic_state3 =
            supports_polygon_mode.then(|| ext::extended_dynamic_state3::Device::new(instance.handle(), &device));
        let max_sampler_anisotropy = unsafe { instance.handle().get_physical_device_properties(physical_device) }
            .limits
            .max_sampler_anisotropy;

        Ok(Self {
            physical_device,
            device,
            queue_family,
            queue,
            max_sampler_anisotropy,
            extended_dynamic_state3,
        })
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

    pub fn extended_dynamic_state3(&self) -> Option<&ext::extended_dynamic_state3::Device> {
        self.extended_dynamic_state3.as_ref()
    }

    fn supports_dynamic_polygon_mode(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> anyhow::Result<bool> {
        let extensions = unsafe { instance.enumerate_device_extension_properties(physical_device) }?;
        let has_extension = extensions
            .iter()
            .any(|extension| extension.extension_name_as_c_str() == Ok(ext::extended_dynamic_state3::NAME));
        if !has_extension {
            return Ok(false);
        }
        let mut extended_dynamic_state3_features = vk::PhysicalDeviceExtendedDynamicState3FeaturesEXT::default();
        let mut features = vk::PhysicalDeviceFeatures2::default().push_next(&mut extended_dynamic_state3_features);
        unsafe { instance.get_physical_device_features2(physical_device, &mut features) };
        Ok(extended_dynamic_state3_features.extended_dynamic_state3_polygon_mode == vk::TRUE)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) };
    }
}
