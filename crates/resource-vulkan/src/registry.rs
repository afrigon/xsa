use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use crate::{AnyEntry, Entry, EntryId, ResourceKind};

type KindEntries<K> = HashMap<(PathBuf, <K as ResourceKind>::Options), Arc<Entry<K>>>;

// Every registered entry, by kind and file for loads and by id for the bookkeeping, and the unreferenced ones kept
// for later, least recently released first.
#[derive(Default)]
pub(crate) struct Registry {
    kinds: HashMap<TypeId, Box<dyn Any + Send>>,
    pub entries: HashMap<EntryId, Arc<dyn AnyEntry>>,
    // An entry released again has a newer sequence, so its older places here are stale.
    pub unreferenced: VecDeque<(u64, EntryId)>,
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

    pub fn keep_unreferenced(&mut self, entry: &dyn AnyEntry) {
        self.next_release += 1;
        entry.set_released_at(self.next_release);
        self.unreferenced.push_back((self.next_release, entry.id()));
    }

    // The unreferenced entries, least recently released first.
    pub fn least_recently_used(&self) -> impl Iterator<Item = &Arc<dyn AnyEntry>> {
        self.unreferenced.iter().filter_map(|(sequence, id)| {
            self.entries
                .get(id)
                .filter(|entry| entry.handles() == 0 && entry.released_at() == *sequence)
        })
    }

    pub fn forget(&mut self, entry: &dyn AnyEntry) {
        entry.forget(self);
    }

    // Drops stale places, so the queue holds each unreferenced entry once.
    pub fn compact(&mut self) {
        let entries = &self.entries;
        self.unreferenced.retain(|(sequence, id)| {
            entries
                .get(id)
                .is_some_and(|entry| entry.handles() == 0 && entry.released_at() == *sequence)
        });
    }
}
