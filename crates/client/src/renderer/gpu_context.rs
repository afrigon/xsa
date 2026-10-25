use memory_vulkan::{BindlessTable, MemoryContext, Timeline, TransferQueue};
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::Window;

use crate::vulkan::{Device, Instance, SuitableDevice, Surface, TransferQueueLocation};

// Every GPU object must drop before this context, so the memory context's last clone, here, destroys the device before
// the surface and the instance. Validation layers report a violation.
pub(super) struct GpuContext {
    pub transfer: TransferQueue,
    pub bindless: BindlessTable,
    pub graphics: Timeline,
    pub encode: Timeline,
    pub memory: MemoryContext,
    pub device: Device,
    pub surface: Surface,
    instance: Instance,
}

impl GpuContext {
    pub fn new(window: &Window) -> anyhow::Result<GpuContext> {
        let instance = Instance::new(window.display_handle()?.as_raw())?;
        let surface = Surface::new(&instance, window)?;
        let suitable = SuitableDevice::select(instance.handle(), &surface)?;
        let memory = Device::create_memory(&instance, &suitable)?;
        let device = Device::new(&instance, &memory, &suitable);
        let graphics = memory.create_timeline(device.queue())?;
        let encode = memory.create_timeline(device.encode_queue())?;
        let transfer_timeline = match device.transfer() {
            TransferQueueLocation::SharedGraphicsQueue => graphics.clone(),
            TransferQueueLocation::DedicatedFamily(_) | TransferQueueLocation::SecondGraphicsQueue => {
                memory.create_timeline(device.transfer_queue())?
            }
        };
        let transfer = TransferQueue::new(
            &memory,
            transfer_timeline,
            device.transfer_family(),
            device.queue_family(),
        )?;
        let bindless = BindlessTable::new(&memory, device.max_sampler_anisotropy())?;

        Ok(GpuContext {
            transfer,
            bindless,
            graphics,
            encode,
            memory,
            device,
            surface,
            instance,
        })
    }

    // Instance-level extension functions, such as the video encoder's capability queries, load through it.
    pub fn entry(&self) -> &ash::Entry {
        self.instance.entry()
    }
}
