use ash::vk;
use memory_vulkan::{DeviceObject, MemoryContext};

use crate::vulkan::Device;

const START: u32 = 0;
const END: u32 = 1;
const QUERY_COUNT: u32 = 2;
const NANOSECONDS_PER_MILLISECOND: f64 = 1e6;

pub(super) struct GpuTimer {
    query_pool: DeviceObject<vk::QueryPool>,
    period_nanoseconds: f32,
}

impl GpuTimer {
    pub fn new(memory: &MemoryContext, device: &Device) -> anyhow::Result<Option<GpuTimer>> {
        let Some(period_nanoseconds) = device.timestamp_period() else {
            return Ok(None);
        };
        let create_info = vk::QueryPoolCreateInfo::default()
            .query_type(vk::QueryType::TIMESTAMP)
            .query_count(QUERY_COUNT);
        let query_pool = DeviceObject::new(memory, unsafe {
            device.handle().create_query_pool(&create_info, None)
        }?);

        Ok(Some(GpuTimer {
            query_pool,
            period_nanoseconds,
        }))
    }

    pub fn begin(&self, device: &Device, command_buffer: vk::CommandBuffer) {
        let device = device.handle();
        let query_pool = self.query_pool.handle();

        unsafe {
            device.cmd_reset_query_pool(command_buffer, query_pool, START, QUERY_COUNT);
            device.cmd_write_timestamp2(command_buffer, vk::PipelineStageFlags2::NONE, query_pool, START);
        }
    }

    pub fn end(&self, device: &Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            device.handle().cmd_write_timestamp2(
                command_buffer,
                vk::PipelineStageFlags2::ALL_COMMANDS,
                self.query_pool.handle(),
                END,
            )
        };
    }

    pub fn milliseconds(&self, device: &Device) -> anyhow::Result<Option<f32>> {
        let mut timestamps = [0_u64; QUERY_COUNT as usize];
        let result = unsafe {
            device.handle().get_query_pool_results(
                self.query_pool.handle(),
                START,
                &mut timestamps,
                vk::QueryResultFlags::TYPE_64,
            )
        };

        match result {
            Ok(()) => {
                let ticks = timestamps[END as usize].wrapping_sub(timestamps[START as usize]);
                let nanoseconds = ticks as f64 * f64::from(self.period_nanoseconds);

                Ok(Some((nanoseconds / NANOSECONDS_PER_MILLISECOND) as f32))
            }
            Err(vk::Result::NOT_READY) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}
