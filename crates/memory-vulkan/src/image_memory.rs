/// Whether an image owns its device memory. A dedicated image's memory is freed with it; a shared image is placed in
/// a block other objects use too, which is freed once all of them are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageMemory {
    Shared,
    Dedicated,
}
