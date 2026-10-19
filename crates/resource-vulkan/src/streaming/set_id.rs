// Names a streamed set for as long as it is registered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SetId {
    pub value: u64,
}
