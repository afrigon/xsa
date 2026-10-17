mod present_fences;

use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::rc::Rc;

use ash::{khr, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator, AllocatorCreateDesc};

use present_fences::PresentFences;

use crate::{Deletion, DeletionQueue, DeviceObject, Frontier, MemoryError, Timeline, TimelineId, TimelineSemaphore};

/// Owns the device, its memory allocator and the queue of objects waiting for the GPU to finish with them.
///
/// Cloning it is cheap: every `DeviceObject` holds a clone, so the device is destroyed only once the last object
/// created from it is gone. Dropping the last clone waits for the GPU, runs every queued deletion, then destroys the
/// allocator and the device. It is single-threaded.
#[derive(Clone)]
pub struct MemoryContext {
    shared: Rc<SharedMemory>,
}

struct SharedMemory {
    device: ash::Device,
    swapchain: khr::swapchain::Device,
    // Dropped by hand, after the queued deletions free their allocations and before the device is destroyed.
    allocator: ManuallyDrop<RefCell<Allocator>>,
    deletions: RefCell<DeletionQueue<Deletion>>,
    timelines: RefCell<Vec<TimelineSemaphore>>,
    presents: RefCell<PresentFences>,
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
            shared: Rc::new(SharedMemory {
                device,
                swapchain,
                allocator: ManuallyDrop::new(RefCell::new(allocator)),
                deletions: RefCell::new(DeletionQueue::default()),
                timelines: RefCell::new(Vec::new()),
                presents: RefCell::new(PresentFences::default()),
            }),
        })
    }

    pub fn device(&self) -> &ash::Device {
        &self.shared.device
    }

    pub fn swapchain_loader(&self) -> &khr::swapchain::Device {
        &self.shared.swapchain
    }

    /// Creates the timeline every submission to `queue` must go through.
    pub fn create_timeline(&self, queue: vk::Queue) -> Result<Timeline, MemoryError> {
        let semaphore = TimelineSemaphore::new(&self.shared.device, queue)?;
        let mut timelines = self.shared.timelines.borrow_mut();
        timelines.push(semaphore);

        Ok(Timeline::new(
            self.clone(),
            TimelineId {
                index: timelines.len() - 1,
            },
        ))
    }

    /// Queues a deletion until the GPU has finished all work submitted so far.
    pub fn defer(&self, deletion: Deletion) {
        let frontier = self.submitted();
        self.shared.deletions.borrow_mut().push(&frontier, deletion);
    }

    /// Runs `action` once the GPU has finished all work submitted so far. The action must not own device objects:
    /// they would keep this context alive from inside its own queue.
    pub fn defer_action(&self, action: impl FnOnce(&ash::Device) + 'static) {
        self.defer(Deletion::Action(Box::new(action)));
    }

    /// Runs every queued deletion the GPU has reached.
    pub fn collect(&self) -> Result<(), MemoryError> {
        let completed = self.completed()?;
        let mut first_error = None;

        // The queue is not borrowed while an entry runs: an action may drop device objects, which queue more.
        while let Some(deletion) = self.next_reached(&completed) {
            if let Err(err) = self.run(deletion) {
                first_error.get_or_insert(err);
            }
        }

        first_error.map_or(Ok(()), Err)
    }

    /// Presents through `queue` with a present fence per swapchain, so the swapchain images and semaphores it uses
    /// are known to be released before the deletion queue destroys them. Returns whether the swapchain is suboptimal.
    pub fn present(&self, queue: vk::Queue, present_info: vk::PresentInfoKHR) -> Result<bool, MemoryError> {
        let device = &self.shared.device;
        let mut presents = self.shared.presents.borrow_mut();
        let mut fences = Vec::with_capacity(present_info.swapchain_count as usize);

        for _ in 0..present_info.swapchain_count {
            match presents.acquire(device) {
                Ok(fence) => fences.push(fence),
                Err(err) => {
                    for fence in fences {
                        presents.release(fence);
                    }

                    return Err(err);
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

        for fence in fences {
            if queued {
                presents.queue(fence);
            } else {
                presents.discard(device, fence);
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
        let allocation = self.shared.allocator.borrow_mut().allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;

        Ok(DeviceObject::new(self, allocation))
    }

    pub(crate) fn submit(
        &self,
        timeline: TimelineId,
        waits: &[vk::SemaphoreSubmitInfo],
        command_buffers: &[vk::CommandBufferSubmitInfo],
        signals: &[vk::SemaphoreSubmitInfo],
    ) -> Result<u64, MemoryError> {
        let (semaphore, queue, value) = {
            let timelines = self.shared.timelines.borrow();
            let state = &timelines[timeline.index];
            (state.semaphore, state.queue, state.submitted + 1)
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
        self.shared.timelines.borrow_mut()[timeline.index].submitted = value;

        Ok(value)
    }

    pub(crate) fn wait_timeline(&self, timeline: TimelineId, value: u64) -> Result<(), MemoryError> {
        let semaphores = [self.shared.timelines.borrow()[timeline.index].semaphore];
        let values = [value];
        let wait_info = vk::SemaphoreWaitInfo::default().semaphores(&semaphores).values(&values);
        unsafe { self.shared.device.wait_semaphores(&wait_info, u64::MAX) }?;

        Ok(())
    }

    fn submitted(&self) -> Frontier {
        Frontier {
            timelines: self
                .shared
                .timelines
                .borrow()
                .iter()
                .map(|timeline| timeline.submitted)
                .collect(),
            presents: self.shared.presents.borrow().queued(),
        }
    }

    fn completed(&self) -> Result<Frontier, MemoryError> {
        let device = &self.shared.device;
        let timelines = self
            .shared
            .timelines
            .borrow()
            .iter()
            .map(|timeline| timeline.completed(device))
            .collect::<Result<_, _>>()?;
        let presents = self.shared.presents.borrow_mut().completed(device)?;

        Ok(Frontier { timelines, presents })
    }

    fn next_reached(&self, completed: &Frontier) -> Option<Deletion> {
        self.shared.deletions.borrow_mut().pop_reached(completed)
    }

    fn run(&self, deletion: Deletion) -> Result<(), MemoryError> {
        self.shared.run(deletion)
    }
}

impl SharedMemory {
    fn run(&self, deletion: Deletion) -> Result<(), MemoryError> {
        let mut allocator = self.allocator.borrow_mut();
        deletion.run(&self.device, &self.swapchain, &mut allocator)
    }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        if let Err(err) = unsafe { self.device.device_wait_idle() } {
            tracing::error!("waiting for the GPU before destroying the device: {err}");
        }

        if let Err(err) = self.presents.get_mut().wait_all(&self.device) {
            tracing::error!("waiting for presents before destroying the device: {err}");
        }

        while let Some(deletion) = self.deletions.get_mut().pop() {
            if let Err(err) = self.run(deletion) {
                tracing::error!("destroying a device object: {err}");
            }
        }

        unsafe {
            for timeline in self.timelines.get_mut().drain(..) {
                self.device.destroy_semaphore(timeline.semaphore, None);
            }

            self.presents.get_mut().destroy(&self.device);
            ManuallyDrop::drop(&mut self.allocator);
            self.device.destroy_device(None);
        }
    }
}
