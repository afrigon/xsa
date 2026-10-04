use std::ffi::c_void;

use anyhow::ensure;
use ash::vk::{self, Handle};

use crate::VulkanHost;

pub(crate) struct VulkanFunctions {
    pub instance: ash::Instance,
    pub device: ash::Device,
    pub physical_device: vk::PhysicalDevice,
}

impl VulkanFunctions {
    pub unsafe fn load(host: &VulkanHost) -> anyhow::Result<VulkanFunctions> {
        ensure!(
            !host.get_instance_proc_addr.is_null(),
            "the host gave no vkGetInstanceProcAddr"
        );

        let get_instance_proc_addr =
            unsafe { std::mem::transmute::<*const c_void, vk::PFN_vkGetInstanceProcAddr>(host.get_instance_proc_addr) };
        let entry = unsafe { ash::Entry::from_static_fn(ash::StaticFn { get_instance_proc_addr }) };
        let instance = unsafe { ash::Instance::load(entry.static_fn(), vk::Instance::from_raw(host.instance)) };
        let device = unsafe { ash::Device::load(instance.fp_v1_0(), vk::Device::from_raw(host.device)) };

        Ok(VulkanFunctions {
            instance,
            device,
            physical_device: vk::PhysicalDevice::from_raw(host.physical_device),
        })
    }
}
