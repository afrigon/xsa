use memory_vulkan::{BindlessTable, TransferPriority};

/// A resource whose uploads are queued, polled by `Resources::update` on the main thread until it is ready.
pub trait PendingResource<K>: Send {
    /// The resource once its uploads complete, with its bindless descriptors written.
    fn poll(&mut self, bindless: &BindlessTable) -> anyhow::Result<Option<K>>;

    /// Moves its uploads that have not started to `priority`.
    fn set_priority(&self, priority: TransferPriority);
}
