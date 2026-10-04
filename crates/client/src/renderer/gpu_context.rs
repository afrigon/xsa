use winit::raw_window_handle::HasDisplayHandle;
use winit::window::Window;

use crate::vulkan::{Allocator, BindlessTextures, Device, Instance, Surface};

pub(super) struct GpuContext {
    pub bindless: BindlessTextures,
    // Declaration order is drop order: allocator before device, device before surface, surface before instance.
    pub allocator: Allocator,
    pub device: Device,
    pub surface: Surface,
    pub instance: Instance,
}

impl GpuContext {
    pub fn new(window: &Window) -> anyhow::Result<GpuContext> {
        let instance = Instance::new(window.display_handle()?.as_raw())?;
        let surface = Surface::new(&instance, window)?;
        let device = Device::new(&instance, &surface)?;
        let allocator = Allocator::new(&instance, &device)?;
        let bindless = BindlessTextures::new(&device)?;

        Ok(GpuContext {
            bindless,
            allocator,
            device,
            surface,
            instance,
        })
    }

    pub fn wait_idle(&self) -> anyhow::Result<()> {
        unsafe { self.device.handle().device_wait_idle() }?;

        Ok(())
    }
}

impl Drop for GpuContext {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.handle().device_wait_idle();
            self.bindless.destroy(&self.device);
        }
    }
}
