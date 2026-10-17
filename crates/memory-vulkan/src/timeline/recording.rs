use ash::vk;

use crate::{MemoryError, RecordingId, Timeline, TimelineValue};

/// A command buffer being recorded for a timeline. While it is open, dropped device objects wait for its submission,
/// so the work it records may use any object alive when the command is recorded. Dropping it without submitting
/// abandons it.
pub struct Recording {
    timeline: Timeline,
    id: RecordingId,
    command_buffer: vk::CommandBuffer,
}

impl Recording {
    pub(crate) fn begin(timeline: &Timeline, command_buffer: vk::CommandBuffer) -> Result<Recording, MemoryError> {
        let recording = Recording {
            timeline: timeline.clone(),
            id: timeline.memory().open_recording(),
            command_buffer,
        };
        let begin_info = vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            timeline
                .memory()
                .device()
                .begin_command_buffer(command_buffer, &begin_info)
        }?;

        Ok(recording)
    }

    pub fn command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    /// Ends the command buffer and submits it to the timeline's queue. Returns the point the submission signals once
    /// it completes.
    pub fn submit(
        self,
        waits: &[vk::SemaphoreSubmitInfo],
        signals: &[vk::SemaphoreSubmitInfo],
    ) -> Result<TimelineValue, MemoryError> {
        unsafe { self.timeline.memory().device().end_command_buffer(self.command_buffer) }?;
        let command_buffers = [vk::CommandBufferSubmitInfo::default().command_buffer(self.command_buffer)];
        let value = self.timeline.submit(waits, &command_buffers, signals)?;

        Ok(TimelineValue {
            timeline: self.timeline.clone(),
            value,
        })
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        self.timeline.memory().close_recording(self.id);
    }
}
