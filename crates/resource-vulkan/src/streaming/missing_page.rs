use crate::streaming::{PageId, PageNeed, PageRetention};

// A wanted page without a layer.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MissingPage {
    pub page: PageId,
    pub score: f32,
    pub need: PageNeed,
    pub retention: PageRetention,
}
