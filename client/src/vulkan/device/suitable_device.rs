use anyhow::Context;
use ash::khr;
use ash::vk;

use crate::vulkan::Surface;

pub(super) struct SuitableDevice {
    pub physical_device: vk::PhysicalDevice,
    pub queue_family: u32,
    pub device_type: vk::PhysicalDeviceType,
}

impl SuitableDevice {
    pub fn select(instance: &ash::Instance, surface: &Surface) -> anyhow::Result<SuitableDevice> {
        let mut suitable = Vec::new();

        for physical_device in unsafe { instance.enumerate_physical_devices() }? {
            if let Some(queue_family) = SuitableDevice::find_queue_family(instance, surface, physical_device)? {
                suitable.push(SuitableDevice {
                    physical_device,
                    queue_family,
                    device_type: unsafe { instance.get_physical_device_properties(physical_device) }.device_type,
                });
            }
        }

        suitable
            .into_iter()
            .min_by_key(|device| device.device_type != vk::PhysicalDeviceType::DISCRETE_GPU)
            .context("no GPU supports Vulkan 1.3 and presenting to this window")
    }

    fn find_queue_family(
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
}
