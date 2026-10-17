mod bookkeeping;
mod present_fences;

use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex, MutexGuard};

use ash::{khr, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator, AllocatorCreateDesc};

use bookkeeping::Bookkeeping;

use crate::{Deletion, DeviceObject, Frontier, MemoryError, RecordingId, Timeline, TimelineId, TimelineSemaphore};

/// Owns the device, its memory allocator and the queue of objects waiting for the GPU to finish with them.
///
/// Cloning it is cheap: every `DeviceObject` holds a clone, so the device is destroyed only once the last object
/// created from it is gone. Dropping the last clone waits for the GPU, runs every queued deletion, then destroys the
/// allocator and the device. It can be shared, and its objects created and dropped, across threads.
#[derive(Clone)]
pub struct MemoryContext {
    shared: Arc<SharedMemory>,
}

// Lock order: a timeline's queue lock before the bookkeeping lock. The allocator lock is never held with another.
struct SharedMemory {
    device: ash::Device,
    swapchain: khr::swapchain::Device,
    // Dropped by hand, after the queued deletions free their allocations and before the device is destroyed.
    allocator: ManuallyDrop<Mutex<Allocator>>,
    bookkeeping: Mutex<Bookkeeping>,
}

impl MemoryContext {
    /// Creates the device described by `create_info`, which must enable timeline semaphores, buffer device addresses
    /// and VK_EXT_swapchain_maintenance1, then its allocator.
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        create_info: &vk::DeviceCreateInfo,
    ) -> Result<MemoryContext, MemoryError> {
        let device = unsafe { instance.create_device(physical_device, create_info, None) }?;
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        });
        let allocator = match allocator {
            Ok(allocator) => allocator,
            Err(err) => {
                unsafe { device.destroy_device(None) };
                return Err(err.into());
            }
        };
        let swapchain = khr::swapchain::Device::new(instance, &device);

        Ok(MemoryContext {
            shared: Arc::new(SharedMemory {
                device,
                swapchain,
                allocator: ManuallyDrop::new(Mutex::new(allocator)),
                bookkeeping: Mutex::new(Bookkeeping::default()),
            }),
        })
    }

    pub fn device(&self) -> &ash::Device {
        &self.shared.device
    }

    pub fn swapchain_loader(&self) -> &khr::swapchain::Device {
        &self.shared.swapchain
    }

    /// Creates the timeline every submission and present to `queue` must go through. A queue has one timeline.
    pub fn create_timeline(&self, queue: vk::Queue) -> Result<Timeline, MemoryError> {
        let semaphore = TimelineSemaphore::new(&self.shared.device)?;
        let mut bookkeeping = self.bookkeeping();
        bookkeeping.timelines.push(semaphore);
        let id = TimelineId {
            index: bookkeeping.timelines.len() - 1,
        };

        Ok(Timeline::new(self.clone(), id, queue))
    }

    /// Queues a deletion until the GPU has finished all work submitted so far, and every recording open now.
    pub fn defer(&self, deletion: Deletion) {
        self.bookkeeping().defer(deletion);
    }

    /// Runs `action` once the GPU has finished all work submitted so far, and every recording open now. The action
    /// must not own device objects: they would keep this context alive from inside its own queue.
    pub fn defer_action(&self, action: impl FnOnce(&ash::Device) + Send + 'static) {
        self.defer(Deletion::Action(Box::new(action)));
    }

    /// Runs every queued deletion the GPU has reached.
    pub fn collect(&self) -> Result<(), MemoryError> {
        let completed = self.bookkeeping().completed(&self.shared.device)?;
        let mut first_error = None;

        // The queue is not locked while an entry runs: an action may drop device objects, which queue more.
        while let Some(deletion) = self.next_reached(&completed) {
            if let Err(err) = self.run(deletion) {
                first_error.get_or_insert(err);
            }
        }

        first_error.map_or(Ok(()), Err)
    }

    pub(crate) fn present(&self, queue: vk::Queue, present_info: vk::PresentInfoKHR) -> Result<bool, MemoryError> {
        let device = &self.shared.device;
        let mut fences = Vec::with_capacity(present_info.swapchain_count as usize);

        {
            let mut bookkeeping = self.bookkeeping();

            for _ in 0..present_info.swapchain_count {
                match bookkeeping.presents.acquire(device) {
                    Ok(fence) => fences.push(fence),
                    Err(err) => {
                        for fence in fences {
                            bookkeeping.presents.release(fence);
                        }

                        return Err(err);
                    }
                }
            }
        }

        let mut fence_info = vk::SwapchainPresentFenceInfoEXT::default().fences(&fences);
        let present_info = present_info.push_next(&mut fence_info);
        let result = unsafe { self.shared.swapchain.queue_present(queue, &present_info) };

        // A present rejected as out of date or with a lost surface is still queued and signals its fence.
        let queued = matches!(
            result,
            Ok(_) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::ERROR_SURFACE_LOST_KHR)
        );
        let mut bookkeeping = self.bookkeeping();

        for fence in fences {
            if queued {
                bookkeeping.presents.queue(fence);
            } else {
                bookkeeping.presents.discard(device, fence);
            }
        }

        Ok(result?)
    }

    pub(crate) fn allocate(
        &self,
        name: &str,
        requirements: vk::MemoryRequirements,
        location: MemoryLocation,
        linear: bool,
    ) -> Result<DeviceObject<Allocation>, MemoryError> {
        let allocation = self.allocator().allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        Ok(DeviceObject::new(self, allocation))
    }

    // The caller holds `queue`'s lock, so values are signalled in the order they are picked.
    pub(crate) fn submit(
        &self,
        timeline: TimelineId,
        queue: vk::Queue,
        waits: &[vk::SemaphoreSubmitInfo],
        command_buffers: &[vk::CommandBufferSubmitInfo],
        signals: &[vk::SemaphoreSubmitInfo],
    ) -> Result<u64, MemoryError> {
        let (semaphore, value) = {
            let bookkeeping = self.bookkeeping();
            let state = &bookkeeping.timelines[timeline.index];
            (state.semaphore, state.submitted + 1)
        };
        let mut all_signals = signals.to_vec();
        all_signals.push(
            vk::SemaphoreSubmitInfo::default()
                .semaphore(semaphore)
                .value(value)
                .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS),
        );
        let submit = vk::SubmitInfo2::default()
            .wait_semaphore_infos(waits)
            .command_buffer_infos(command_buffers)
            .signal_semaphore_infos(&all_signals);
        unsafe { self.shared.device.queue_submit2(queue, &[submit], vk::Fence::null()) }?;
        self.bookkeeping().timelines[timeline.index].submitted = value;

        Ok(value)
    }

    pub(crate) fn open_recording(&self) -> RecordingId {
        self.bookkeeping().recordings.open()
    }

    pub(crate) fn close_recording(&self, id: RecordingId) {
        self.bookkeeping().close_recording(id);
    }

    pub(crate) fn wait_timeline(&self, timeline: TimelineId, value: u64) -> Result<(), MemoryError> {
        let semaphores = [self.bookkeeping().timelines[timeline.index].semaphore];
        let values = [value];
        let wait_info = vk::SemaphoreWaitInfo::default().semaphores(&semaphores).values(&values);
        unsafe { self.shared.device.wait_semaphores(&wait_info, u64::MAX) }?;

        Ok(())
    }

    fn next_reached(&self, completed: &Frontier) -> Option<Deletion> {
        self.bookkeeping().deletions.pop_reached(completed)
    }

    fn bookkeeping(&self) -> MutexGuard<'_, Bookkeeping> {
        self.shared
            .bookkeeping
            .lock()
            .expect("the bookkeeping lock is poisoned")
    }

    fn allocator(&self) -> MutexGuard<'_, Allocator> {
        self.shared.allocator.lock().expect("the allocator lock is poisoned")
    }

    fn run(&self, deletion: Deletion) -> Result<(), MemoryError> {
        self.shared.run(deletion)
    }
}

impl SharedMemory {
    fn run(&self, deletion: Deletion) -> Result<(), MemoryError> {
        let mut allocator = self.allocator.lock().expect("the allocator lock is poisoned");
        deletion.run(&self.device, &self.swapchain, &mut allocator)
    }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        if let Err(err) = unsafe { self.device.device_wait_idle() } {
            tracing::error!("waiting for the GPU before destroying the device: {err}");
        }

        let bookkeeping = self.bookkeeping.get_mut().expect("the bookkeeping lock is poisoned");
        let allocator = self.allocator.get_mut().expect("the allocator lock is poisoned");

        if let Err(err) = bookkeeping.presents.wait_all(&self.device) {
            tracing::error!("waiting for presents before destroying the device: {err}");
        }

        while let Some(deletion) = bookkeeping.deletions.pop() {
            if let Err(err) = deletion.run(&self.device, &self.swapchain, allocator) {
                tracing::error!("destroying a device object: {err}");
            }
        }

        unsafe {
            for timeline in bookkeeping.timelines.drain(..) {
                self.device.destroy_semaphore(timeline.semaphore, None);
            }

            bookkeeping.presents.destroy(&self.device);
            ManuallyDrop::drop(&mut self.allocator);
            self.device.destroy_device(None);
        }
    }
}
