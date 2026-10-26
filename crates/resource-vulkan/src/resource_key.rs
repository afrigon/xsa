use std::path::PathBuf;

// What makes two loads the same resource: the file, read with the same options.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct ResourceKey<O> {
    pub path: PathBuf,
    pub options: O,
}
