use anyhow::Context;
use ash::vk;
use ash::{ext, khr};

use super::TransferQueueLocation;
use crate::vulkan::Surface;

#[derive(Clone, Copy)]
pub struct SuitableDevice {
    pub physical_device: vk::PhysicalDevice,
    pub queue_family: u32,
    pub transfer: TransferQueueLocation,
    pub device_type: vk::PhysicalDeviceType,
    // Wireframe needs the polygon mode to be dynamic state (VK_EXT_extended_dynamic_state3).
    pub dynamic_polygon_mode: bool,
}

impl SuitableDevice {
    pub fn select(instance: &ash::Instance, surface: &Surface) -> anyhow::Result<SuitableDevice> {
        let mut suitable = Vec::new();

        for physical_device in unsafe { instance.enumerate_physical_devices() }? {
            if let Some(queue_family) = SuitableDevice::find_queue_family(instance, surface, physical_device)? {
                let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
                suitable.push(SuitableDevice {
                    physical_device,
                    queue_family,
                    transfer: TransferQueueLocation::select(&queue_families, queue_family),
                    device_type: unsafe { instance.get_physical_device_properties(physical_device) }.device_type,
                    dynamic_polygon_mode: SuitableDevice::supports_dynamic_polygon_mode(instance, physical_device)?,
                });
            }
        }

        suitable
            .into_iter()
            .min_by_key(|device| device.device_type != vk::PhysicalDeviceType::DISCRETE_GPU)
            .context("no GPU supports Vulkan 1.3, swapchain maintenance and presenting to this window")
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
        let supports = |name| {
            extensions
                .iter()
                .any(|extension| extension.extension_name_as_c_str() == Ok(name))
        };

        if !supports(khr::swapchain::NAME) || !supports(ext::swapchain_maintenance1::NAME) {
            return Ok(None);
        }

        let mut swapchain_maintenance1 = vk::PhysicalDeviceSwapchainMaintenance1FeaturesEXT::default();
        let mut features = vk::PhysicalDeviceFeatures2::default().push_next(&mut swapchain_maintenance1);
        unsafe { instance.get_physical_device_features2(physical_device, &mut features) };

        if swapchain_maintenance1.swapchain_maintenance1 != vk::TRUE {
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
