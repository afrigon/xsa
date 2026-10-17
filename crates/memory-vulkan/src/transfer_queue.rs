mod batch;
mod batch_builder;
mod copy_rate;
mod copy_region;
mod gpu_upload;
mod image_copy;
mod image_upload;
mod owned_upload_completion;
mod pending_requests;
mod piece;
mod piece_copy;
mod readback_completion;
mod readback_ticket;
mod request_id;
mod staged_readback;
mod staging_block;
mod staging_ring;
mod staging_slice;
mod submission_thread;
mod texel_block;
mod ticket_slot;
mod transfer_command;
mod transfer_completion;
mod transfer_priority;
mod transfer_request;
mod transfer_shared;
mod transfer_status;
mod transfer_ticket;
mod transfer_work;
mod upload_destination;
mod upload_target;
mod upload_ticket;

pub use gpu_upload::GpuUpload;
pub use image_copy::ImageCopy;
pub use image_upload::ImageUpload;
pub use readback_ticket::ReadbackTicket;
pub use texel_block::TexelBlock;
pub use transfer_priority::TransferPriority;
pub use transfer_status::TransferStatus;
pub use upload_ticket::UploadTicket;

pub(crate) use batch::Batch;
pub(crate) use batch_builder::BatchBuilder;
pub(crate) use copy_rate::CopyRate;
pub(crate) use copy_region::CopyRegion;
pub(crate) use owned_upload_completion::OwnedUploadCompletion;
pub(crate) use pending_requests::PendingRequests;
pub(crate) use piece::Piece;
pub(crate) use piece_copy::PieceCopy;
pub(crate) use readback_completion::ReadbackCompletion;
pub(crate) use request_id::RequestId;
pub(crate) use staged_readback::StagedReadback;
pub(crate) use staging_block::StagingBlock;
pub(crate) use staging_ring::{STAGING_BLOCK_SIZE, StagingRing};
pub(crate) use staging_slice::StagingSlice;
pub(crate) use submission_thread::SubmissionThread;
pub(crate) use ticket_slot::TicketSlot;
pub(crate) use transfer_command::TransferCommand;
pub(crate) use transfer_completion::TransferCompletion;
pub(crate) use transfer_request::TransferRequest;
pub(crate) use transfer_shared::TransferShared;
pub(crate) use transfer_ticket::TransferTicket;
pub(crate) use transfer_work::TransferWork;
pub(crate) use upload_destination::UploadDestination;
pub(crate) use upload_target::UploadTarget;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use ash::vk;

use crate::{Buffer, Image, MemoryContext, MemoryError, QueueSharing, Timeline, TimelineValue};

/// Every copy between the CPU and the GPU: uploads into images and buffers, and readbacks. A thread of its own
/// records and submits requests in priority order to the transfer queue, through host-visible staging that grows
/// while copies wait and shrinks once they are done.
///
/// Cloning it is cheap, and clones share the queue across threads. The queue stops once the last clone and the last
/// ticket are gone.
#[derive(Clone)]
pub struct TransferQueue {
    handle: Arc<QueueHandle>,
}

// Stops the submission thread when the last clone drops.
struct QueueHandle {
    shared: Arc<TransferShared>,
    thread: Option<JoinHandle<()>>,
}

impl TransferQueue {
    /// Starts the queue submitting to `timeline`'s queue, of `transfer_family`. When it is the graphics queue itself,
    /// `timeline` is the graphics timeline.
    pub fn new(
        memory: &MemoryContext,
        timeline: Timeline,
        transfer_family: u32,
        graphics_family: u32,
    ) -> Result<TransferQueue, MemoryError> {
        let sharing = if transfer_family == graphics_family {
            QueueSharing::Exclusive
        } else {
            QueueSharing::Concurrent(vec![graphics_family, transfer_family])
        };
        let shared = Arc::new(TransferShared::new(memory, timeline, transfer_family, sharing)?);
        let submission = SubmissionThread::new(Arc::clone(&shared))?;
        let thread = std::thread::Builder::new()
            .name("transfer".to_owned())
            .spawn(move || submission.run())
            .expect("the transfer thread can be spawned");

        Ok(TransferQueue {
            handle: Arc::new(QueueHandle {
                shared,
                thread: Some(thread),
            }),
        })
    }

    /// The sharing the images and buffers it copies to or from must be created with, so both the graphics and the
    /// transfer queue may use them.
    pub fn sharing(&self) -> QueueSharing {
        self.shared().sharing.clone()
    }

    /// Writes `upload` into `image`, from undefined contents to ready for sampling in every level and layer.
    pub fn upload_image(&self, image: Image, upload: ImageUpload, priority: TransferPriority) -> UploadTicket<Image> {
        let target = UploadTarget::Image {
            image: image.handle(),
            range: image.subresource_range(),
        };
        let pieces = Piece::split_image(&upload);

        self.upload(image, target, upload.data, pieces, priority)
    }

    /// Writes `data` at the start of `buffer`.
    pub fn upload_buffer(&self, buffer: Buffer, data: Vec<u8>, priority: TransferPriority) -> UploadTicket<Buffer> {
        let target = UploadTarget::Buffer(buffer.handle());
        let pieces = if data.len() as u64 > buffer.size() {
            Err(MemoryError::HostAccess {
                buffer: "upload data".to_owned(),
                reason: "it is larger than its destination",
            })
        } else {
            Ok(Piece::split_bytes(data.len()))
        };

        self.upload(buffer, target, vec![data], pieces, priority)
    }

    /// Copies all of `buffer` back to the CPU, once the GPU has reached `after` if given. The buffer is created with
    /// `TRANSFER_SRC` usage and this queue's sharing.
    pub fn read_buffer(
        &self,
        buffer: Buffer,
        after: Option<TimelineValue>,
        priority: TransferPriority,
    ) -> ReadbackTicket {
        let size = buffer.size() as usize;
        let result = Arc::new(Mutex::new(vec![0; size]));
        let slot = Arc::new(TicketSlot::pending());
        let work = TransferWork::Readback {
            source: buffer.handle(),
            after,
            result: Arc::clone(&result),
        };
        let completion = ReadbackCompletion::new(buffer, result, Arc::downgrade(&slot));
        let id = self.shared().next_request_id();
        self.shared().enqueue(TransferRequest {
            id,
            priority,
            work,
            pieces: Piece::split_bytes(size),
            started: false,
            completion: Box::new(completion),
        });

        ReadbackTicket::new(TransferTicket::new(slot, id, self.clone()))
    }

    /// A wait for every copy whose ticket was seen complete, for the submissions that use their results. Its value
    /// is already signalled, so it never delays the GPU.
    pub fn visibility_wait(&self) -> vk::SemaphoreSubmitInfo<'static> {
        let shared = self.shared();

        vk::SemaphoreSubmitInfo::default()
            .semaphore(shared.timeline_semaphore)
            .value(shared.observed())
            .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
    }

    fn upload<T: Send + 'static>(
        &self,
        destination: T,
        target: UploadTarget,
        data: Vec<Vec<u8>>,
        pieces: Result<VecDeque<Piece>, MemoryError>,
        priority: TransferPriority,
    ) -> UploadTicket<T> {
        let slot = Arc::new(TicketSlot::pending());
        let completion = Box::new(OwnedUploadCompletion::new(destination, Arc::downgrade(&slot)));
        let id = self.shared().next_request_id();

        match pieces {
            Ok(pieces) => self.shared().enqueue(TransferRequest {
                id,
                priority,
                work: TransferWork::Upload {
                    data,
                    destination: UploadDestination::Owned(target),
                },
                pieces,
                started: false,
                completion,
            }),
            Err(err) => completion.fail(err),
        }

        UploadTicket::new(TransferTicket::new(slot, id, self.clone()))
    }

    fn reprioritize(&self, id: RequestId, priority: TransferPriority) {
        self.shared().reprioritize(id, priority);
    }

    fn shared(&self) -> &TransferShared {
        &self.handle.shared
    }
}

impl Drop for QueueHandle {
    fn drop(&mut self) {
        {
            let mut requests = self.shared.requests();
            requests.shutdown = true;
            self.shared.wake(&mut requests);
        }

        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            tracing::error!("the transfer thread panicked");
        }
    }
}
