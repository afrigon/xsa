use anyhow::Context;
use ash::vk;
use ash::{ext, khr};

use super::{Instance, Surface};

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
        let (physical_device, queue_family) = select_physical_device(instance.handle(), surface)?;

        let queue_priorities = [1.0];
        let queue_create_infos = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family)
            .queue_priorities(&queue_priorities)];
        let supports_polygon_mode = supports_dynamic_polygon_mode(instance.handle(), physical_device)?;
        let mut extensions = vec![khr::swapchain::NAME.as_ptr()];
        if supports_polygon_mode {
            extensions.push(ext::extended_dynamic_state3::NAME.as_ptr());
        }
        let features = vk::PhysicalDeviceFeatures::default()
            .fill_mode_non_solid(true)
            .geometry_shader(true)
            .sampler_anisotropy(true);
        let mut vulkan_11_features = vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true);
        let mut vulkan_12_features = vk::PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .runtime_descriptor_array(true)
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_sampled_image_update_after_bind(true)
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

fn supports_dynamic_polygon_mode(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> anyhow::Result<bool> {
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
