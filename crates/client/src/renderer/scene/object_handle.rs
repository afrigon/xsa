#[derive(Clone, Copy, PartialEq)]
pub struct ObjectHandle {
    pub(in crate::renderer) index: usize,
}
