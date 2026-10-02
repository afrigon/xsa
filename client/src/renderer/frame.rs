use ash::vk;

use super::MATERIAL_CAPACITY;
use super::exposure::HISTOGRAM_BINS;
use super::gpu_data::{FrameData, ObjectData};
use super::material::MaterialData;
use crate::vulkan::{Allocator, Buffer, Device, MemoryLocation};

pub(super) struct Frame {
    pub command_pool: vk::CommandPool,
    pub command_buffer: vk::CommandBuffer,
    pub image_acquired: vk::Semaphore,
    pub in_flight: vk::Fence,
    pub frame_data: Buffer,
    pub objects: Buffer,
    pub materials: Buffer,
    pub histogram: Buffer,
    pub histogram_ready: bool,
}

impl Frame {
    pub fn new(device: &Device, allocator: &mut Allocator, object_capacity: usize) -> anyhow::Result<Self> {
        let frame_data = create_shader_buffer(device, allocator, "frame data", size_of::<FrameData>())?;
        let objects = create_object_buffer(device, allocator, object_capacity)?;
        let materials = create_shader_buffer(
            device,
            allocator,
            "material data",
            MATERIAL_CAPACITY * size_of::<MaterialData>(),
        )?;

        let histogram = Buffer::new(
            device,
            allocator,
            "luminance histogram",
            (HISTOGRAM_BINS * size_of::<u32>()) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuToCpu,
        )?;

        let queue_family = device.queue_family();
        let device = device.handle();
        let pool_info = vk::CommandPoolCreateInfo::default().queue_family_index(queue_family);
        let command_pool = unsafe { device.create_command_pool(&pool_info, None) }?;
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info) }?[0];
        let image_acquired = unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }?;
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let in_flight = unsafe { device.create_fence(&fence_info, None) }?;

        Ok(Self {
            command_pool,
            command_buffer,
            image_acquired,
            in_flight,
            frame_data,
            objects,
            materials,
            histogram,
            histogram_ready: false,
        })
    }

    pub unsafe fn destroy(&mut self, device: &Device, allocator: &mut Allocator) {
        unsafe {
            self.frame_data.destroy(device, allocator);
            self.objects.destroy(device, allocator);
            self.materials.destroy(device, allocator);
            self.histogram.destroy(device, allocator);
            let device = device.handle();
            device.destroy_fence(self.in_flight, None);
            device.destroy_semaphore(self.image_acquired, None);
            device.destroy_command_pool(self.command_pool, None);
        }
    }
}

pub(super) fn create_object_buffer(
    device: &Device,
    allocator: &mut Allocator,
    capacity: usize,
) -> anyhow::Result<Buffer> {
    create_shader_buffer(device, allocator, "object data", capacity * size_of::<ObjectData>())
}

fn create_shader_buffer(device: &Device, allocator: &mut Allocator, name: &str, size: usize) -> anyhow::Result<Buffer> {
    Buffer::new(
        device,
        allocator,
        name,
        size as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        MemoryLocation::CpuToGpu,
    )
}
