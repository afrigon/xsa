use memory_vulkan::BindlessTable;

use crate::{EntryId, LoadPriority, MemoryUsage, Registry, ResourceState};

// An entry of any kind, as the bookkeeping sees it.
pub(crate) trait AnyEntry: Send + Sync {
    fn id(&self) -> EntryId;

    // Bytes of its file, known before it loads.
    fn size(&self) -> u64;

    fn state(&self) -> ResourceState;

    fn acquire(&self);

    fn release(&self);

    fn handles(&self) -> usize;

    fn is_cached(&self) -> bool;

    fn is_required(&self) -> bool;

    fn released_at(&self) -> u64;

    fn set_released_at(&self, sequence: u64);

    // Zero until it is ready.
    fn memory(&self) -> MemoryUsage;

    // Polls its pending uploads; true once it settled.
    fn poll(&self, bindless: &BindlessTable) -> bool;

    fn set_priority(&self, priority: LoadPriority);

    // Removes it from the registry; a load still running for it is skipped or its result dropped.
    fn forget(&self, registry: &mut Registry);

    // Its file and who asked for it, for messages.
    fn describe(&self) -> String;

    fn is_settled(&self) -> bool {
        !matches!(self.state(), ResourceState::Loading)
    }
}
