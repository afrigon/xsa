//! GPU memory and object lifetime for Vulkan.
//!
//! A `MemoryContext` owns the device, its memory allocator and a deletion queue. Every object created from the device
//! is wrapped in a `DeviceObject` as soon as it exists; dropping it queues its destruction until the GPU has finished
//! all work submitted so far, which every submission reports through a `Timeline` and every present through a present
//! fence. Work is recorded through a timeline's `Recording`: an object dropped while one is open waits for its
//! submission too. Plain actions, such as returning a slot to a free list, wait in the same queue. Every type can be
//! shared across threads.

mod buffer;
mod deletion_queue;
mod device_handle;
mod device_object;
mod image;
mod image_description;
mod memory_context;
mod memory_error;
mod queue_sharing;
mod timeline;

pub(crate) use deletion_queue::{DeletionQueue, Frontier, OpenRecordings, RecordingId};
pub(crate) use timeline::{TimelineId, TimelineSemaphore};

pub use buffer::Buffer;
pub use deletion_queue::Deletion;
pub use device_handle::DeviceHandle;
pub use device_object::DeviceObject;
pub use gpu_allocator::MemoryLocation;
pub use image::Image;
pub use image_description::ImageDescription;
pub use memory_context::MemoryContext;
pub use memory_error::MemoryError;
pub use queue_sharing::QueueSharing;
pub use timeline::{Recording, Timeline};

#[cfg(test)]
mod tests {
    use ash::vk;

    use super::*;

    fn assert_thread_safe<T: Send + Sync>() {}

    #[test]
    fn device_objects_cross_threads() {
        assert_thread_safe::<MemoryContext>();
        assert_thread_safe::<Timeline>();
        assert_thread_safe::<Recording>();
        assert_thread_safe::<DeviceObject<vk::Buffer>>();
        assert_thread_safe::<Buffer>();
        assert_thread_safe::<Image>();
    }
}
