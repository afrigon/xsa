use memory_vulkan::{BindlessTable, MemoryContext, TransferPriority, TransferQueue};

/// What a kind loads with on a worker thread.
pub struct LoadContext<'a> {
    pub memory: &'a MemoryContext,
    pub transfer: &'a TransferQueue,
    pub bindless: &'a BindlessTable,
    /// Who asked for the resource, for the GPU objects' debug names.
    pub name: &'a str,
    pub priority: TransferPriority,
}
