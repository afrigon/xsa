// Names a request so its ticket can find it while it waits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RequestId {
    pub value: u64,
}
