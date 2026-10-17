use memory_vulkan::{BindlessTable, MemoryContext, TransferQueue};

// The GPU objects every load creates its resource with.
pub(crate) struct LoadEnvironment {
    pub memory: MemoryContext,
    pub transfer: TransferQueue,
    pub bindless: BindlessTable,
}
