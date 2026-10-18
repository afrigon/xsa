// What a wanted page without a layer needs next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PageNeed {
    Read,
    Layer,
}
