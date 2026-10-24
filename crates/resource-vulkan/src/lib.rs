//! Background, reference-counted loading of the files game content refers to.
//!
//! `Resources` loads each file once on a worker pool of its own, hands out typed handles at once, and frees a
//! resource once its last handle is dropped, or later when memory is needed. Each kind of file is a `ResourceKind`;
//! this crate provides textures and cube maps, and users add their own.
//!
//! Paged data larger than memory streams through a `StreamedSet`: its client requests the pages it wants each frame,
//! and the pages stay resident in texture array chunks as far as the shared memory budget allows.

mod admission;
mod any_entry;
mod any_handle;
mod available_memory;
mod entry;
mod entry_id;
mod events;
mod eviction_plan;
mod eviction_rank;
mod group_progress;
mod load_context;
mod load_environment;
mod load_priority;
mod load_request;
mod memory_budget;
mod memory_room;
mod memory_usage;
mod pending_resource;
mod registry;
mod resource_group;
mod resource_handle;
mod resource_key;
mod resource_kind;
mod resource_limits;
mod resource_state;
mod resources;
mod retention;
mod soft_owner;
mod soft_usage;
mod streaming;
mod textures;
mod unavailable;
mod unreferenced_entry;
mod worker_pool;

pub(crate) use admission::Admission;
pub(crate) use any_entry::AnyEntry;
pub(crate) use any_handle::AnyHandle;
pub(crate) use available_memory::AvailableMemory;
pub(crate) use entry::Entry;
pub(crate) use entry_id::EntryId;
pub(crate) use events::Events;
pub(crate) use eviction_plan::EvictionPlan;
pub(crate) use eviction_rank::EvictionRank;
pub(crate) use load_environment::LoadEnvironment;
pub(crate) use memory_budget::{BudgetReading, MemoryBudget};
pub(crate) use memory_room::MemoryRoom;
pub(crate) use registry::Registry;
pub(crate) use resource_key::ResourceKey;
pub(crate) use soft_owner::SoftOwner;
pub(crate) use soft_usage::SoftUsage;
pub(crate) use unavailable::Unavailable;
pub(crate) use unreferenced_entry::UnreferencedEntry;
pub(crate) use worker_pool::{Job, JobOwner, WorkerPool};

pub use group_progress::GroupProgress;
pub use load_context::LoadContext;
pub use load_priority::LoadPriority;
pub use load_request::LoadRequest;
pub use memory_usage::MemoryUsage;
pub use pending_resource::PendingResource;
pub use resource_group::ResourceGroup;
pub use resource_handle::ResourceHandle;
pub use resource_kind::ResourceKind;
pub use resource_limits::ResourceLimits;
pub use resource_state::ResourceState;
pub use resources::Resources;
pub use retention::Retention;
pub use streaming::{PageFormat, PagePlacement, PageRequest, PageSource, Residency, SetStatistics, StreamedSet};
pub use textures::{CubeMap, PendingTexture, Texture, TextureArray};

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_thread_safe<T: Send + Sync>() {}

    #[test]
    fn resources_cross_threads() {
        assert_thread_safe::<Resources>();
        assert_thread_safe::<ResourceHandle<Texture>>();
        assert_thread_safe::<ResourceGroup>();
    }
}
