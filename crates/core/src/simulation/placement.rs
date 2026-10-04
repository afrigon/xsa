use super::offset::Offset;

pub(super) struct Placement {
    pub parent: Option<usize>,
    pub offset: Offset,
}
