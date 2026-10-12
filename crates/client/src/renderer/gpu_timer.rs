use ash::vk;

use crate::vulkan::Device;

const START: u32 = 0;
const END: u32 = 1;
const QUERY_COUNT: u32 = 2;
const NANOSECONDS_PER_MILLISECOND: f64 = 1e6;

pub(super) struct GpuTimer {
    query_pool: vk::QueryPool,
    period_nanoseconds: f32,
}

impl GpuTimer {
    pub fn new(device: &Device) -> anyhow::Result<Option<GpuTimer>> {
        let Some(period_nanoseconds) = device.timestamp_period() else {
            return Ok(None);
        };
        let create_info = vk::QueryPoolCreateInfo::default()
            .query_type(vk::QueryType::TIMESTAMP)
            .query_count(QUERY_COUNT);
        let query_pool = unsafe { device.handle().create_query_pool(&create_info, None) }?;

        Ok(Some(GpuTimer {
            query_pool,
            period_nanoseconds,
        }))
    }

    pub fn begin(&self, device: &Device, command_buffer: vk::CommandBuffer) {
        let device = device.handle();

        unsafe {
            device.cmd_reset_query_pool(command_buffer, self.query_pool, START, QUERY_COUNT);
            device.cmd_write_timestamp2(command_buffer, vk::PipelineStageFlags2::NONE, self.query_pool, START);
        }
    }

    pub fn end(&self, device: &Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            device.handle().cmd_write_timestamp2(
                command_buffer,
                vk::PipelineStageFlags2::ALL_COMMANDS,
                self.query_pool,
                END,
            )
        };
    }

    pub fn milliseconds(&self, device: &Device) -> anyhow::Result<Option<f32>> {
        let mut timestamps = [0_u64; QUERY_COUNT as usize];
        let result = unsafe {
            device.handle().get_query_pool_results(
                self.query_pool,
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

    // Safety: only once the GPU has finished every frame that used the timer.
    pub unsafe fn destroy(&mut self, device: &Device) {
        unsafe { device.handle().destroy_query_pool(self.query_pool, None) };
    }
}
