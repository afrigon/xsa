use std::sync::Arc;

use crate::{AnyEntry, AnyHandle, GroupProgress, MipHandle, ResourceHandle, ResourceKind, StreamedMips};

/// A set of resources owned by whoever needs them together, such as a scene: its progress, and one place to raise
/// their priority. It holds its resources as a handle would.
#[derive(Default)]
pub struct ResourceGroup {
    members: Vec<AnyHandle>,
}

impl ResourceGroup {
    pub fn new() -> ResourceGroup {
        ResourceGroup::default()
    }

    pub fn add<K: ResourceKind>(&mut self, handle: &ResourceHandle<K>) {
        self.members
            .push(AnyHandle::new(Arc::clone(&handle.entry) as Arc<dyn AnyEntry>));
    }

    pub fn add_mips<K: StreamedMips>(&mut self, handle: &MipHandle<K>) {
        self.members
            .push(AnyHandle::new(Arc::clone(&handle.entry) as Arc<dyn AnyEntry>));
    }

    pub fn progress(&self) -> GroupProgress {
        self.members.iter().fold(GroupProgress::default(), |progress, member| {
            let settled = member.entry.is_settled();

            GroupProgress {
                loaded_bytes: progress.loaded_bytes + if settled { member.entry.size() } else { 0 },
                total_bytes: progress.total_bytes + member.entry.size(),
                loading: progress.loading + usize::from(!settled),
            }
        })
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = &Arc<dyn AnyEntry>> {
        self.members.iter().map(|member| &member.entry)
    }
}
