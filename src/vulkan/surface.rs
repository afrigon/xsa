use anyhow::{Context, bail};
use ash::khr;
use ash::vk;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use winit::window::Window;

use super::Instance;

pub struct Surface {
    loader: khr::surface::Instance,
    surface: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(instance: &Instance, window: &Window) -> anyhow::Result<Self> {
        let display_handle = window.display_handle()?.as_raw();
        let window_handle = window.window_handle()?.as_raw();
        let surface = create_platform_surface(instance, display_handle, window_handle)
            .context("creating the Vulkan surface")?;
        Ok(Self {
            loader: khr::surface::Instance::new(instance.entry(), instance.handle()),
            surface,
        })
    }

    pub fn handle(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn loader(&self) -> &khr::surface::Instance {
        &self.loader
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_surface(self.surface, None) };
    }
}

fn create_platform_surface(
    instance: &Instance,
    display_handle: RawDisplayHandle,
    window_handle: RawWindowHandle,
) -> anyhow::Result<vk::SurfaceKHR> {
    let entry = instance.entry();
    let instance = instance.handle();
    let surface = match (display_handle, window_handle) {
        (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(window)) => {
            let create_info = vk::WaylandSurfaceCreateInfoKHR::default()
                .display(display.display.as_ptr())
                .surface(window.surface.as_ptr());
            let loader = khr::wayland_surface::Instance::new(entry, instance);
            unsafe { loader.create_wayland_surface(&create_info, None) }
        }
        (RawDisplayHandle::Xlib(display), RawWindowHandle::Xlib(window)) => {
            let connection = display.display.context("the Xlib display handle is missing its connection")?;
            let create_info = vk::XlibSurfaceCreateInfoKHR::default()
                .dpy(connection.as_ptr())
                .window(window.window);
            let loader = khr::xlib_surface::Instance::new(entry, instance);
            unsafe { loader.create_xlib_surface(&create_info, None) }
        }
        (RawDisplayHandle::Xcb(display), RawWindowHandle::Xcb(window)) => {
            let connection = display.connection.context("the XCB display handle is missing its connection")?;
            let create_info = vk::XcbSurfaceCreateInfoKHR::default()
                .connection(connection.as_ptr())
                .window(window.window.get());
            let loader = khr::xcb_surface::Instance::new(entry, instance);
            unsafe { loader.create_xcb_surface(&create_info, None) }
        }
        (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(window)) => {
            let create_info = vk::Win32SurfaceCreateInfoKHR::default()
                .hwnd(window.hwnd.get())
                .hinstance(window.hinstance.map_or(0, |hinstance| hinstance.get()));
            let loader = khr::win32_surface::Instance::new(entry, instance);
            unsafe { loader.create_win32_surface(&create_info, None) }
        }
        (display, window) => bail!("unsupported window system: {display:?} / {window:?}"),
    };
    Ok(surface?)
}
