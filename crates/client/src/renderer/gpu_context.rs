use memory_vulkan::{MemoryContext, Timeline};
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::Window;

use crate::vulkan::{BindlessTextures, Device, Instance, SuitableDevice, Surface};

// Every GPU object must drop before this context, so the memory context's last clone, here, destroys the device before
// the surface and the instance. Validation layers report a violation.
pub(super) struct GpuContext {
    pub bindless: BindlessTextures,
    pub graphics: Timeline,
    pub memory: MemoryContext,
    pub device: Device,
    pub surface: Surface,
    _instance: Instance,
}

impl GpuContext {
    pub fn new(window: &Window) -> anyhow::Result<GpuContext> {
        let instance = Instance::new(window.display_handle()?.as_raw())?;
        let surface = Surface::new(&instance, window)?;
        let suitable = SuitableDevice::select(instance.handle(), &surface)?;
        let memory = Device::create_memory(&instance, &suitable)?;
        let device = Device::new(&instance, &memory, &suitable);
        let graphics = memory.create_timeline(device.queue())?;
        let bindless = BindlessTextures::new(&memory, &device)?;

        Ok(GpuContext {
            bindless,
            graphics,
            memory,
            device,
            surface,
            _instance: instance,
        })
    }
}
