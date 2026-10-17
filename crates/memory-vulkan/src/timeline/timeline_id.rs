/// A timeline's index among its context's timelines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TimelineId {
    pub index: usize,
}
