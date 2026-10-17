use std::path::PathBuf;

use crate::ResourceKind;

// What makes two loads the same resource: the file, read with the same options.
pub(crate) struct ResourceKey<K: ResourceKind> {
    pub path: PathBuf,
    pub options: K::Options,
}

impl<K: ResourceKind> Clone for ResourceKey<K> {
    fn clone(&self) -> ResourceKey<K> {
        ResourceKey {
            path: self.path.clone(),
            options: self.options.clone(),
        }
    }
}

impl<K: ResourceKind> PartialEq for ResourceKey<K> {
    fn eq(&self, other: &ResourceKey<K>) -> bool {
        self.path == other.path && self.options == other.options
    }
}

impl<K: ResourceKind> Eq for ResourceKey<K> {}

impl<K: ResourceKind> std::hash::Hash for ResourceKey<K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self.options.hash(state);
    }
}
