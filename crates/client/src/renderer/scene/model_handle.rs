#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ModelHandle {
    pub(in crate::renderer) index: usize,
}
