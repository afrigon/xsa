#[derive(Clone, Copy, PartialEq)]
pub struct LightSourceHandle {
    pub(in crate::renderer) index: usize,
}
