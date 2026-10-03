mod debug_messenger;

use std::ffi::{CStr, c_char};

use anyhow::{Context, bail};
use ash::ext::debug_utils;
use ash::khr;
use ash::vk;
use winit::raw_window_handle::RawDisplayHandle;

use debug_messenger::DebugMessenger;

const VALIDATION_LAYER: &CStr = c"VK_LAYER_KHRONOS_validation";

pub struct Instance {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_messenger: Option<DebugMessenger>,
}

impl Instance {
    pub fn new(display_handle: RawDisplayHandle) -> anyhow::Result<Self> {
        let entry = unsafe { ash::Entry::load() }.context("loading the Vulkan library")?;
        let validation = cfg!(debug_assertions);

        let mut extensions: Vec<*const c_char> = Instance::surface_extensions(display_handle)?
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
        let instance = unsafe { entry.create_instance(&create_info, None) }.context("creating the Vulkan instance")?;

        let mut vulkan = Self {
            entry,
            instance,
            debug_messenger: None,
        };

        if validation {
            vulkan.debug_messenger = Some(DebugMessenger::new(&vulkan.entry, &vulkan.instance)?);
        }

        Ok(vulkan)
    }

    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn handle(&self) -> &ash::Instance {
        &self.instance
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
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            if let Some(debug_messenger) = &self.debug_messenger {
                debug_messenger.destroy();
            }

            self.instance.destroy_instance(None);
        }
    }
}
