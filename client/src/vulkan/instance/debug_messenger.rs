use std::ffi::{CStr, c_void};

use anyhow::Context;
use ash::ext::debug_utils;
use ash::vk;

pub(super) struct DebugMessenger {
    loader: debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> anyhow::Result<DebugMessenger> {
        let loader = debug_utils::Instance::new(entry, instance);
        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(log_debug_message));
        let messenger = unsafe { loader.create_debug_utils_messenger(&create_info, None) }
            .context("creating the Vulkan debug messenger")?;
        Ok(DebugMessenger { loader, messenger })
    }

    // Safety: only while the instance that created it is alive.
    pub unsafe fn destroy(&self) {
        unsafe { self.loader.destroy_debug_utils_messenger(self.messenger, None) };
    }
}

unsafe extern "system" fn log_debug_message(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let message = unsafe { CStr::from_ptr((*callback_data).p_message) };
    eprintln!("[vulkan {severity:?} {message_type:?}] {}", message.to_string_lossy());
    vk::FALSE
}
