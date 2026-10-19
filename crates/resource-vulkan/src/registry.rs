use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use crate::{AnyEntry, Entry, EntryId, ResourceKey, ResourceKind, SoftOwner, SoftUsage, UnreferencedEntry};

type KindEntries<K> = HashMap<ResourceKey<K>, Arc<Entry<K>>>;

// Every registered entry, by kind and file for loads and by id for the bookkeeping, and the unreferenced ones kept
// for later, least recently released first.
#[derive(Default)]
pub(crate) struct Registry {
    kinds: HashMap<TypeId, Box<dyn Any + Send>>,
    pub entries: HashMap<EntryId, Arc<dyn AnyEntry>>,
    pub unreferenced: VecDeque<UnreferencedEntry>,
    next_entry: u64,
    next_release: u64,
}

impl Registry {
    pub fn kind<K: ResourceKind>(&mut self) -> &mut KindEntries<K> {
        self.kinds
            .entry(TypeId::of::<K>())
            .or_insert_with(|| Box::new(KindEntries::<K>::new()))
            .downcast_mut()
            .expect("a kind's entries are stored under its type id")
    }

    pub fn next_id(&mut self) -> EntryId {
        self.next_entry += 1;
        EntryId { value: self.next_entry }
    }

    pub fn insert<K: ResourceKind>(&mut self, entry: Arc<Entry<K>>) {
        self.entries.insert(entry.id(), Arc::clone(&entry) as Arc<dyn AnyEntry>);
        self.kind::<K>().insert(entry.key(), entry);
    }

    pub fn keep_unreferenced(&mut self, entry: &dyn AnyEntry, now: u64) {
        self.next_release += 1;
        entry.set_released_at(self.next_release);
        self.unreferenced.push_back(UnreferencedEntry {
            sequence: self.next_release,
            entry: entry.id(),
            released_in: now,
        });
    }

    // The unreferenced entries' memory, least recently released first.
    pub fn unreferenced_usage(&self) -> impl Iterator<Item = SoftUsage> + '_ {
        self.unreferenced.iter().filter_map(|unreferenced| {
            let entry = self.current(unreferenced)?;

            Some(SoftUsage {
                owner: SoftOwner::Entry(entry.id()),
                memory: entry.memory(),
                last_use: unreferenced.released_in,
                score: 0.0,
            })
        })
    }

    pub fn forget(&mut self, entry: &dyn AnyEntry) {
        entry.forget(self);
    }

    // Drops stale places, so the queue holds each unreferenced entry once.
    pub fn compact(&mut self) {
        let unreferenced = std::mem::take(&mut self.unreferenced);
        self.unreferenced = unreferenced
            .into_iter()
            .filter(|unreferenced| self.current(unreferenced).is_some())
            .collect();
    }

    // The entry, if this is still its place: unreferenced since this release.
    fn current(&self, unreferenced: &UnreferencedEntry) -> Option<&Arc<dyn AnyEntry>> {
        self.entries
            .get(&unreferenced.entry)
            .filter(|entry| entry.handles() == 0 && entry.released_at() == unreferenced.sequence)
    }
}
