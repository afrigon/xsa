use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::transfer_queue::{
    Batch, BatchBuilder, CopyRate, CopyRegion, PieceCopy, StagedReadback, StagingRing, TransferCommand,
    TransferRequest, TransferShared, TransferWork, UploadDestination, UploadTarget,
};
use crate::{DeviceObject, MemoryError};

// How long a Critical request may wait behind copies already submitted: one frame at 60 Hz.
const LATENCY_BUDGET: Duration = Duration::from_micros(16_667);

// Records and submits requests in priority order, and settles their tickets as the transfer timeline advances.
//
// Requests wait as owned bytes until their turn, so priorities hold: the copies submitted ahead of a new request are
// bounded by the latency budget at the measured copy rate, with at least one piece in flight. The thread takes a
// request from the shared queue only when it is more urgent than the one it is working through.
pub(crate) struct SubmissionThread {
    shared: Arc<TransferShared>,
    command_pool: DeviceObject<vk::CommandPool>,
    free_command_buffers: Vec<vk::CommandBuffer>,
    uploads: StagingRing,
    readbacks: StagingRing,
    started: Vec<TransferRequest>,
    in_flight: VecDeque<Batch>,
    copy_rate: CopyRate,
    seen_wake: u64,
}

impl SubmissionThread {
    pub fn new(shared: Arc<TransferShared>) -> Result<SubmissionThread, MemoryError> {
        let memory = &shared.memory;
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(shared.family);
        let command_pool = DeviceObject::new(memory, unsafe {
            memory.device().create_command_pool(&pool_info, None)
        }?);
        let uploads = StagingRing::new(
            memory,
            "upload staging",
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        );
        let readbacks = StagingRing::new(
            memory,
            "readback staging",
            vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuToCpu,
        );

        Ok(SubmissionThread {
            shared,
            command_pool,
            free_command_buffers: Vec::new(),
            uploads,
            readbacks,
            started: Vec::new(),
            in_flight: VecDeque::new(),
            copy_rate: CopyRate::default(),
            seen_wake: 0,
        })
    }

    pub fn run(mut self) {
        loop {
            {
                let requests = self.shared.requests();

                if requests.shutdown {
                    return;
                }

                self.seen_wake = requests.wake_value;
            }

            if let Err(err) = self.step().and_then(|()| self.wait()) {
                tracing::error!("the transfer queue stopped: {err}");
                self.stop(err);
                return;
            }
        }
    }

    fn step(&mut self) -> Result<(), MemoryError> {
        self.complete_batches()?;

        if self.in_flight.is_empty() && self.started.is_empty() && self.shared.requests().is_empty() {
            self.uploads.shrink();
            self.readbacks.shrink();
        }

        while self.submit_batch() {}

        Ok(())
    }

    fn wait(&self) -> Result<(), MemoryError> {
        let mut semaphores = vec![self.shared.wake.handle()];
        let mut values = vec![self.seen_wake + 1];

        if let Some(oldest) = self.in_flight.front() {
            semaphores.push(self.shared.timeline_semaphore);
            values.push(oldest.value);
        }

        let wait_info = vk::SemaphoreWaitInfo::default()
            .flags(vk::SemaphoreWaitFlags::ANY)
            .semaphores(&semaphores)
            .values(&values);
        unsafe { self.shared.memory.device().wait_semaphores(&wait_info, u64::MAX) }?;

        Ok(())
    }

    fn complete_batches(&mut self) -> Result<(), MemoryError> {
        let completed = self.shared.completed()?;
        let completed_at = Instant::now();
        let mut finished = Vec::new();

        while self.in_flight.front().is_some_and(|batch| batch.value <= completed) {
            finished.extend(self.in_flight.pop_front());
        }

        let (Some(first), Some(last)) = (finished.first(), finished.last()) else {
            return Ok(());
        };

        let bytes = finished.iter().map(|batch| batch.bytes).sum();
        self.copy_rate.measure(bytes, first.submitted_at, completed_at);
        self.shared.observe(last.value);

        for batch in finished {
            self.finish(batch)?;
        }

        Ok(())
    }

    fn finish(&mut self, batch: Batch) -> Result<(), MemoryError> {
        for readback in &batch.readbacks {
            let mut result = readback.result.lock().expect("a readback's lock is poisoned");
            let destination = &mut result[readback.result_offset..][..readback.size];
            self.readbacks
                .buffer(readback.slice.block)
                .read_at(readback.slice.offset, destination)?;
        }

        self.uploads.release(&batch.upload_blocks);
        self.readbacks.release(&batch.readback_blocks);
        self.free_command_buffers.push(batch.command_buffer);

        for completion in batch.completions {
            completion.complete();
        }

        Ok(())
    }

    // Builds and submits one batch; false when there was nothing to submit.
    fn submit_batch(&mut self) -> bool {
        let budget = self.copy_rate.bytes_in(LATENCY_BUDGET);
        let in_flight: u64 = self.in_flight.iter().map(|batch| batch.bytes).sum();
        let mut builder = BatchBuilder::default();

        loop {
            self.take_more_urgent_requests();

            let Some(request) = self.started.first() else {
                break;
            };

            if request.completion.is_cancelled() {
                self.started.remove(0);
                continue;
            }

            let queued = in_flight + builder.bytes;

            if let Some(piece) = request.pieces.front()
                && queued > 0
                && queued + piece.size as u64 > budget
            {
                break;
            }

            if let Err(err) = self.stage_next_piece(&mut builder) {
                self.fail_batch(builder, err);
                return false;
            }
        }

        if builder.is_empty() {
            return false;
        }

        match self.record_and_submit(&builder) {
            Ok((value, command_buffer)) => {
                self.in_flight.push_back(Batch {
                    value,
                    command_buffer,
                    bytes: builder.bytes,
                    submitted_at: Instant::now(),
                    upload_blocks: self.uploads.seal(),
                    readback_blocks: self.readbacks.seal(),
                    readbacks: builder.readbacks,
                    completions: builder.completions,
                });
                true
            }
            Err(err) => {
                self.fail_batch(builder, err);
                false
            }
        }
    }

    fn take_more_urgent_requests(&mut self) {
        let mut requests = self.shared.requests();

        while let Some(priority) = requests.highest_priority()
            && self.started.first().is_none_or(|first| priority < first.priority)
        {
            self.started.extend(requests.pop());
            self.started.rotate_right(1);
        }
    }

    fn stage_next_piece(&mut self, builder: &mut BatchBuilder) -> Result<(), MemoryError> {
        let SubmissionThread {
            started,
            uploads,
            readbacks,
            ..
        } = self;
        let request = &mut started[0];

        if let Some(piece) = request.pieces.pop_front() {
            if !request.started {
                builder.begin(request);
                request.started = true;
            }

            match &request.work {
                TransferWork::Upload { data, destination } => {
                    for copy in &piece.copies {
                        let slice = uploads.allocate(copy.size, copy.alignment)?;
                        let bytes = &(*data[copy.data]).as_ref()[copy.data_offset..][..copy.size];
                        let staging = uploads.buffer_mut(slice.block);
                        staging.write_at(slice.offset, bytes)?;
                        let command =
                            SubmissionThread::upload_command(staging.handle(), slice.offset, destination, copy);
                        builder.copy(command, copy.size);
                    }
                }
                TransferWork::Readback { source, after, result } => {
                    if let Some(after) = after {
                        builder.wait_for(after);
                    }

                    for copy in &piece.copies {
                        let CopyRegion::Buffer { offset } = copy.region else {
                            unreachable!("a readback copies from a buffer");
                        };
                        let slice = readbacks.allocate(copy.size, copy.alignment)?;
                        let command = TransferCommand::BufferToBuffer {
                            source: *source,
                            destination: readbacks.buffer(slice.block).handle(),
                            region: vk::BufferCopy {
                                src_offset: offset,
                                dst_offset: slice.offset as u64,
                                size: copy.size as u64,
                            },
                        };
                        builder.copy(command, copy.size);
                        builder.readbacks.push(StagedReadback {
                            slice,
                            size: copy.size,
                            result: Arc::clone(result),
                            result_offset: copy.data_offset,
                        });
                    }
                }
            }
        }

        if request.pieces.is_empty() {
            builder.finish(started.remove(0));
        }

        Ok(())
    }

    fn upload_command(
        staging: vk::Buffer,
        staging_offset: usize,
        destination: &UploadDestination,
        copy: &PieceCopy,
    ) -> TransferCommand {
        let (target, copy_layout) = match destination {
            UploadDestination::Owned(target) => (target, vk::ImageLayout::TRANSFER_DST_OPTIMAL),
            UploadDestination::Region(target) => (target, vk::ImageLayout::GENERAL),
        };

        match (*target, copy.region) {
            (
                UploadTarget::Image { image, range, .. },
                CopyRegion::Image {
                    mip_level,
                    array_layer,
                    offset,
                    extent,
                },
            ) => TransferCommand::BufferToImage {
                source: staging,
                image,
                layout: copy_layout,
                region: vk::BufferImageCopy::default()
                    .buffer_offset(staging_offset as u64)
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(range.aspect_mask)
                            .mip_level(mip_level)
                            .base_array_layer(array_layer)
                            .layer_count(1),
                    )
                    .image_offset(vk::Offset3D {
                        x: offset.x,
                        y: offset.y,
                        z: 0,
                    })
                    .image_extent(extent.into()),
            },
            (UploadTarget::Buffer(buffer), CopyRegion::Buffer { offset }) => TransferCommand::BufferToBuffer {
                source: staging,
                destination: buffer,
                region: vk::BufferCopy {
                    src_offset: staging_offset as u64,
                    dst_offset: offset,
                    size: copy.size as u64,
                },
            },
            _ => unreachable!("an upload's copies match its destination"),
        }
    }

    fn record_and_submit(&mut self, builder: &BatchBuilder) -> Result<(u64, vk::CommandBuffer), MemoryError> {
        let command_buffer = self.command_buffer()?;
        let recording = self.shared.timeline.begin(command_buffer)?;
        unsafe { builder.record(self.shared.memory.device(), command_buffer) };
        let submitted = recording.submit(&builder.semaphore_waits(), &[]);

        match submitted {
            Ok(point) => Ok((point.value, command_buffer)),
            Err(err) => {
                self.free_command_buffers.push(command_buffer);
                Err(err)
            }
        }
    }

    fn command_buffer(&mut self) -> Result<vk::CommandBuffer, MemoryError> {
        if let Some(command_buffer) = self.free_command_buffers.pop() {
            return Ok(command_buffer);
        }

        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool.handle())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        Ok(unsafe { self.shared.memory.device().allocate_command_buffers(&allocate_info) }?[0])
    }

    // A batch that cannot be built or submitted fails, and so does every request in progress: their copies would be
    // left incomplete.
    fn fail_batch(&mut self, builder: BatchBuilder, error: MemoryError) {
        tracing::warn!("a transfer batch failed: {error}");
        self.uploads.abandon();
        self.readbacks.abandon();
        let error = Arc::new(error);
        let started = std::mem::take(&mut self.started);

        for completion in builder
            .completions
            .into_iter()
            .chain(started.into_iter().map(|request| request.completion))
        {
            completion.fail(MemoryError::Transfer(Arc::clone(&error)));
        }
    }

    fn stop(&mut self, error: MemoryError) {
        let error = Arc::new(error);
        let mut requests = self.shared.requests();
        requests.failure = Some(Arc::clone(&error));
        let in_flight = self.in_flight.drain(..).flat_map(|batch| batch.completions);
        let started = self.started.drain(..).map(|request| request.completion);
        let pending = requests.drain().map(|request| request.completion);

        for completion in in_flight.chain(started).chain(pending) {
            completion.fail(MemoryError::Transfer(Arc::clone(&error)));
        }
    }
}
