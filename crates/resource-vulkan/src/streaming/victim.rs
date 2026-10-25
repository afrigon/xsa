use crate::streaming::PageOwner;

// A page whose cell a more important page may take.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Victim {
    pub owner: PageOwner,
    pub last_use: u64,
    // Its score while it is wanted.
    pub score: Option<f32>,
}
