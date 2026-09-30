use std::ffi::{CStr, c_char, c_void};

use anyhow::{Context, bail};
use ash::ext::debug_utils;
use ash::khr;
use ash::vk;
use winit::raw_window_handle::RawDisplayHandle;

const VALIDATION_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";

pub struct Instance {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_messenger: Option<(debug_utils::Instance, vk::DebugUtilsMessengerEXT)>,
}

impl Instance {
    pub fn new(display_handle: RawDisplayHandle) -> anyhow::Result<Self> {
        let entry = unsafe { ash::Entry::load() }.context("loading the Vulkan library")?;
        let validation = cfg!(debug_assertions);

        let mut extensions: Vec<*const c_char> = surface_extensions(display_handle)?
            .iter()
            .map(|name| name.as_ptr())
            .collect();
        let mut layers = Vec::new();
        if validation {
            extensions.push(debug_utils::NAME.as_ptr());
            layers.push(VALIDATION_LAYER.as_ptr());
        }

        let application_info = vk::ApplicationInfo::default()
            .application_name(c"xsa")
            .api_version(vk::API_VERSION_1_3);
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions);
        let instance = unsafe { entry.create_instance(&create_info, None) }
            .context("creating the Vulkan instance")?;

        let mut vulkan = Self {
            entry,
            instance,
            debug_messenger: None,
        };
        if validation {
            vulkan.debug_messenger = Some(create_debug_messenger(&vulkan.entry, &vulkan.instance)?);
        }
        Ok(vulkan)
    }

    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn handle(&self) -> &ash::Instance {
        &self.instance
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            if let Some((loader, messenger)) = &self.debug_messenger {
                loader.destroy_debug_utils_messenger(*messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

fn surface_extensions(display_handle: RawDisplayHandle) -> anyhow::Result<[&'static CStr; 2]> {
    let platform_extension = match display_handle {
        RawDisplayHandle::Wayland(_) => khr::wayland_surface::NAME,
        RawDisplayHandle::Xlib(_) => khr::xlib_surface::NAME,
        RawDisplayHandle::Xcb(_) => khr::xcb_surface::NAME,
        RawDisplayHandle::Windows(_) => khr::win32_surface::NAME,
        other => bail!("unsupported display server: {other:?}"),
    };
    Ok([khr::surface::NAME, platform_extension])
}

fn create_debug_messenger(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> anyhow::Result<(debug_utils::Instance, vk::DebugUtilsMessengerEXT)> {
    let loader = debug_utils::Instance::new(entry, instance);
    let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(log_debug_message));
    let messenger = unsafe { loader.create_debug_utils_messenger(&create_info, None) }
        .context("creating the Vulkan debug messenger")?;
    Ok((loader, messenger))
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
