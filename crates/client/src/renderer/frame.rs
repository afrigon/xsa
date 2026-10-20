use ash::vk;
use memory_vulkan::{Buffer, DeviceObject, MemoryContext, MemoryLocation, QueueSharing, TimelineValue};

use super::gpu_data::{FrameData, ObjectData, ProjectedLightData, StarData, SurfaceAnchorData, TileData, TilePages};
use super::gpu_timer::GpuTimer;
use super::star_visibility::HISTOGRAM_BINS;
use crate::vulkan::Device;

pub(super) struct Frame {
    pub command_pool: DeviceObject<vk::CommandPool>,
    pub command_buffer: vk::CommandBuffer,
    pub image_acquired: DeviceObject<vk::Semaphore>,
    pub frame_data: Buffer,
    pub objects: Buffer,
    pub tiles: Buffer,
    pub tile_pages: Buffer,
    pub stars: Buffer,
    pub projected_lights: Buffer,
    pub surface_anchors: Buffer,
    pub histogram: Buffer,
    pub timer: Option<GpuTimer>,
    // The graphics timeline value of the last frame submitted from this slot; once it is reached, the slot may be
    // recorded again and its read-backs hold that frame's results.
    pub submitted: Option<TimelineValue>,
}

impl Frame {
    pub fn new(
        memory: &MemoryContext,
        device: &Device,
        object_capacity: usize,
        tile_capacity: usize,
        star_capacity: usize,
        light_capacity: usize,
        surface_anchor_capacity: usize,
    ) -> anyhow::Result<Self> {
        let frame_data = Frame::create_shader_buffer(memory, "frame data", size_of::<FrameData>())?;
        let objects = Frame::create_object_buffer(memory, object_capacity)?;
        let tiles = Frame::create_tile_buffer(memory, tile_capacity)?;
        let tile_pages = Frame::create_tile_pages_buffer(memory, tile_capacity)?;
        let stars = Frame::create_star_buffer(memory, star_capacity)?;
        let projected_lights = Frame::create_projected_light_buffer(memory, light_capacity)?;
        let surface_anchors = Frame::create_surface_anchor_buffer(memory, surface_anchor_capacity)?;

        let histogram = Buffer::new(
            memory,
            "luminance histogram",
            (HISTOGRAM_BINS * size_of::<u32>()) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuToCpu,
            &QueueSharing::Exclusive,
        )?;

        let timer = GpuTimer::new(memory, device)?;
        let queue_family = device.queue_family();
        let device = device.handle();
        let pool_info = vk::CommandPoolCreateInfo::default().queue_family_index(queue_family);
        let command_pool = DeviceObject::new(memory, unsafe { device.create_command_pool(&pool_info, None) }?);
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool.handle())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info) }?[0];
        let image_acquired = DeviceObject::new(memory, unsafe {
            device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
        }?);

        Ok(Self {
            command_pool,
            command_buffer,
            image_acquired,
            frame_data,
            objects,
            tiles,
            tile_pages,
            stars,
            projected_lights,
            surface_anchors,
            histogram,
            timer,
            submitted: None,
        })
    }

    pub fn create_object_buffer(memory: &MemoryContext, capacity: usize) -> anyhow::Result<Buffer> {
        Frame::create_shader_buffer(memory, "object data", capacity * size_of::<ObjectData>())
    }

    pub fn create_tile_buffer(memory: &MemoryContext, capacity: usize) -> anyhow::Result<Buffer> {
        Frame::create_shader_buffer(memory, "tile data", capacity * size_of::<TileData>())
    }

    pub fn create_tile_pages_buffer(memory: &MemoryContext, capacity: usize) -> anyhow::Result<Buffer> {
        Frame::create_shader_buffer(memory, "tile pages", capacity * size_of::<TilePages>())
    }

    pub fn create_star_buffer(memory: &MemoryContext, capacity: usize) -> anyhow::Result<Buffer> {
        Frame::create_shader_buffer(memory, "star data", capacity * size_of::<StarData>())
    }

    pub fn create_projected_light_buffer(memory: &MemoryContext, capacity: usize) -> anyhow::Result<Buffer> {
        Frame::create_shader_buffer(
            memory,
            "projected light data",
            capacity * size_of::<ProjectedLightData>(),
        )
    }

    pub fn create_surface_anchor_buffer(memory: &MemoryContext, capacity: usize) -> anyhow::Result<Buffer> {
        Frame::create_shader_buffer(memory, "surface anchors", capacity * size_of::<SurfaceAnchorData>())
    }

    fn create_shader_buffer(memory: &MemoryContext, name: &str, size: usize) -> anyhow::Result<Buffer> {
        Ok(Buffer::new(
            memory,
            name,
            size as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            MemoryLocation::CpuToGpu,
            &QueueSharing::Exclusive,
        )?)
    }
}
