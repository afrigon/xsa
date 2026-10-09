use crate::HistoryId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ImageOwner {
    Transient,
    History { id: HistoryId },
}
