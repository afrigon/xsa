/// Whether a set's resident pages follow its requests. A frozen set keeps the pages it holds: nothing loads or is
/// evicted while requests keep arriving.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Residency {
    #[default]
    Live,
    Frozen,
}
