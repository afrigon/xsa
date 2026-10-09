/// An imported image in the frame being built. Unlike an `ImageHandle`, it has no `ImageId`: the graph does not own
/// it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportedImageHandle {
    pub(crate) index: usize,
}
