use std::ffi::c_void;

/// The host's Vulkan objects, as raw 64-bit handle values so a host on any Vulkan binding can provide them.
///
/// Every Vulkan function is loaded through `get_instance_proc_addr`, the host's `vkGetInstanceProcAddr`. The
/// device must have Vulkan 1.3's dynamic rendering and synchronization2 enabled.
pub struct VulkanHost {
    pub get_instance_proc_addr: *const c_void,
    pub instance: u64,
    pub physical_device: u64,
    pub device: u64,
}
