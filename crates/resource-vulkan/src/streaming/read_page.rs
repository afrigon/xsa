use crate::streaming::{PageOwner, ReadResult};

// A read a worker finished, for the streamer's next update.
pub(crate) struct ReadPage {
    pub owner: PageOwner,
    pub result: ReadResult,
}
