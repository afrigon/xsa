/// A pair of images kept across frames, created once with `RenderGraph::create_history`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryId {
    pub(crate) index: usize,
}
