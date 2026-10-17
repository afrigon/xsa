// An amount of memory, written with a unit in the config (`512MiB`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ByteSize {
    pub bytes: u64,
}
