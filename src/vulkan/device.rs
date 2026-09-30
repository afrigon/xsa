use anyhow::Context;
use ash::khr;
use ash::vk;

use super::{Instance, Surface};

pub struct Device {
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    queue_family: u32,
    queue: vk::Queue,
}

impl Device {
    pub fn new(instance: &Instance, surface: &Surface) -> anyhow::Result<Self> {
        let (physical_device, queue_family) = select_physical_device(instance.handle(), surface)?;

        let queue_priorities = [1.0];
        let queue_create_infos = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family)
            .queue_priorities(&queue_priorities)];
        let extensions = [khr::swapchain::NAME.as_ptr()];
        let mut vulkan_11_features = vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true);
        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);
        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extensions)
            .push_next(&mut vulkan_11_features)
            .push_next(&mut vulkan_13_features);
        let device = unsafe { instance.handle().create_device(physical_device, &create_info, None) }
            .context("creating the Vulkan device")?;
        let queue = unsafe { device.get_device_queue(queue_family, 0) };

        Ok(Self {
            physical_device,
            device,
            queue_family,
            queue,
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
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) };
    }
}

fn select_physical_device(
    instance: &ash::Instance,
    surface: &Surface,
) -> anyhow::Result<(vk::PhysicalDevice, u32)> {
    let mut suitable = Vec::new();
    for physical_device in unsafe { instance.enumerate_physical_devices() }? {
        if let Some(queue_family) = suitable_queue_family(instance, surface, physical_device)? {
            let device_type = unsafe { instance.get_physical_device_properties(physical_device) }.device_type;
            suitable.push((physical_device, queue_family, device_type));
        }
    }
    let (physical_device, queue_family, _) = suitable
        .into_iter()
        .min_by_key(|(_, _, device_type)| *device_type != vk::PhysicalDeviceType::DISCRETE_GPU)
        .context("no GPU supports Vulkan 1.3 and presenting to this window")?;
    Ok((physical_device, queue_family))
}

fn suitable_queue_family(
    instance: &ash::Instance,
    surface: &Surface,
    physical_device: vk::PhysicalDevice,
) -> anyhow::Result<Option<u32>> {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    if properties.api_version < vk::API_VERSION_1_3 {
        return Ok(None);
    }

    let extensions = unsafe { instance.enumerate_device_extension_properties(physical_device) }?;
    let supports_swapchain = extensions
        .iter()
        .any(|extension| extension.extension_name_as_c_str() == Ok(khr::swapchain::NAME));
    if !supports_swapchain {
        return Ok(None);
    }

    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    for (index, queue_family) in (0..).zip(&queue_families) {
        let presents = unsafe {
            surface
                .loader()
                .get_physical_device_surface_support(physical_device, index, surface.handle())
        }?;
        if presents && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            return Ok(Some(index));
        }
    }
    Ok(None)
}
