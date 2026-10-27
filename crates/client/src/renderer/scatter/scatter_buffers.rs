use ash::vk;
use memory_vulkan::{Buffer, MemoryContext, MemoryError, MemoryLocation, QueueSharing};

use super::{DRAW_ARGUMENTS_BYTES, MESH_INSTANCE_BYTES, ScatterFrame};
use crate::renderer::gpu_data::{ScatterDrawData, ScatterModelData, ScatterPageData, ScatterUseData};

const INITIAL_PAGES: usize = 256;
const INITIAL_USES: usize = 4;
const INITIAL_MODELS: usize = 8;
const INITIAL_LISTS: usize = 8;
const INITIAL_DRAWS: usize = 16;
const INITIAL_INSTANCES: usize = 16_384;
const COUNTER_BYTES: u64 = size_of::<u32>() as u64;
const SHADER_USAGE: vk::BufferUsageFlags = vk::BufferUsageFlags::from_raw(
    vk::BufferUsageFlags::STORAGE_BUFFER.as_raw() | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS.as_raw(),
);

// One frame slot's scatter buffers. The records the CPU writes each frame; the instance lists, their counters and the
// draws' indirect arguments the GPU writes; and the counters copied back, read once the slot's frame completed. A
// buffer outgrown is created again at the next power of two.
pub(in crate::renderer) struct ScatterBuffers {
    pub pages: Buffer,
    pub uses: Buffer,
    pub models: Buffer,
    pub draws: Buffer,
    pub instances: Buffer,
    // One counter per list, then the count of instances within range.
    pub counts: Buffer,
    pub arguments: Buffer,
    pub readback: Buffer,
    // The triangles each instance of the slot's last frame's lists draws, by list.
    pub recorded_lists: Vec<u64>,
}

impl ScatterBuffers {
    pub fn new(memory: &MemoryContext) -> anyhow::Result<ScatterBuffers> {
        Ok(ScatterBuffers {
            pages: ScatterBuffers::records::<ScatterPageData>(memory, "scatter pages", INITIAL_PAGES)?,
            uses: ScatterBuffers::records::<ScatterUseData>(memory, "scatter uses", INITIAL_USES)?,
            models: ScatterBuffers::records::<ScatterModelData>(memory, "scatter models", INITIAL_MODELS)?,
            draws: ScatterBuffers::records::<ScatterDrawData>(memory, "scatter draws", INITIAL_DRAWS)?,
            instances: ScatterBuffers::instances(memory, INITIAL_INSTANCES)?,
            counts: ScatterBuffers::counts(memory, INITIAL_LISTS)?,
            arguments: ScatterBuffers::arguments(memory, INITIAL_DRAWS)?,
            readback: ScatterBuffers::readback(memory, INITIAL_LISTS)?,
            recorded_lists: Vec::new(),
        })
    }

    // Grows what the frame outgrew, then writes its records.
    pub fn write(&mut self, memory: &MemoryContext, frame: &ScatterFrame) -> anyhow::Result<()> {
        if frame.pages.len() as u64 * size_of::<ScatterPageData>() as u64 > self.pages.size() {
            let capacity = frame.pages.len().next_power_of_two();
            self.pages = ScatterBuffers::records::<ScatterPageData>(memory, "scatter pages", capacity)?;
        }

        if frame.uses.len() as u64 * size_of::<ScatterUseData>() as u64 > self.uses.size() {
            let capacity = frame.uses.len().next_power_of_two();
            self.uses = ScatterBuffers::records::<ScatterUseData>(memory, "scatter uses", capacity)?;
        }

        if frame.models.len() as u64 * size_of::<ScatterModelData>() as u64 > self.models.size() {
            let capacity = frame.models.len().next_power_of_two();
            self.models = ScatterBuffers::records::<ScatterModelData>(memory, "scatter models", capacity)?;
        }

        if frame.draws.len() as u64 * size_of::<ScatterDrawData>() as u64 > self.draws.size() {
            let capacity = frame.draws.len().next_power_of_two();
            self.draws = ScatterBuffers::records::<ScatterDrawData>(memory, "scatter draws", capacity)?;
            self.arguments = ScatterBuffers::arguments(memory, capacity)?;
        }

        if u64::from(frame.instance_count) * MESH_INSTANCE_BYTES > self.instances.size() {
            let capacity = (frame.instance_count as usize).next_power_of_two();
            self.instances = ScatterBuffers::instances(memory, capacity)?;
        }

        if (frame.lists.len() as u64 + 1) * COUNTER_BYTES > self.counts.size() {
            let capacity = (frame.lists.len() + 1).next_power_of_two();
            self.counts = ScatterBuffers::counts(memory, capacity)?;
            self.readback = ScatterBuffers::readback(memory, capacity)?;
        }

        self.pages.write(&frame.pages)?;
        self.uses.write(&frame.uses)?;
        self.models.write(&frame.models)?;
        self.draws.write(&frame.draws)?;
        self.recorded_lists.clear();
        self.recorded_lists
            .extend(frame.lists.iter().map(|list| list.triangles_per_instance));

        Ok(())
    }

    // The counters of the slot's last frame: one per list, then the count within range.
    pub fn read_counts(&self) -> Result<Vec<u32>, MemoryError> {
        let mut counts = vec![0; self.recorded_lists.len() + 1];
        self.readback.read(&mut counts)?;

        Ok(counts)
    }

    pub fn counts_size(&self) -> u64 {
        (self.recorded_lists.len() as u64 + 1) * COUNTER_BYTES
    }

    fn records<T>(memory: &MemoryContext, name: &str, capacity: usize) -> anyhow::Result<Buffer> {
        ScatterBuffers::buffer(
            memory,
            name,
            capacity as u64 * size_of::<T>() as u64,
            SHADER_USAGE,
            MemoryLocation::CpuToGpu,
        )
    }

    fn instances(memory: &MemoryContext, capacity: usize) -> anyhow::Result<Buffer> {
        let size = capacity as u64 * MESH_INSTANCE_BYTES;
        ScatterBuffers::buffer(memory, "mesh instances", size, SHADER_USAGE, MemoryLocation::GpuOnly)
    }

    fn counts(memory: &MemoryContext, capacity: usize) -> anyhow::Result<Buffer> {
        let usage = SHADER_USAGE | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::TRANSFER_SRC;
        ScatterBuffers::buffer(
            memory,
            "scatter counts",
            capacity as u64 * COUNTER_BYTES,
            usage,
            MemoryLocation::GpuOnly,
        )
    }

    fn arguments(memory: &MemoryContext, capacity: usize) -> anyhow::Result<Buffer> {
        let usage = SHADER_USAGE | vk::BufferUsageFlags::INDIRECT_BUFFER;
        let size = capacity as u64 * DRAW_ARGUMENTS_BYTES;
        ScatterBuffers::buffer(memory, "scatter draw arguments", size, usage, MemoryLocation::GpuOnly)
    }

    fn readback(memory: &MemoryContext, capacity: usize) -> anyhow::Result<Buffer> {
        let size = capacity as u64 * COUNTER_BYTES;
        let usage = vk::BufferUsageFlags::TRANSFER_DST;
        ScatterBuffers::buffer(memory, "scatter counts readback", size, usage, MemoryLocation::GpuToCpu)
    }

    fn buffer(
        memory: &MemoryContext,
        name: &str,
        size: u64,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
    ) -> anyhow::Result<Buffer> {
        Ok(Buffer::new(
            memory,
            name,
            size,
            usage,
            location,
            &QueueSharing::Exclusive,
        )?)
    }
}
