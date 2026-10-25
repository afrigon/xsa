/// A page a set wants this frame, and how much it matters: the on-screen error in pixels while it stays on its
/// coarser fallback. Pages load from the highest score down, across every set.
#[derive(Clone, Copy, Debug)]
pub struct PageRequest<K> {
    pub key: K,
    pub score: f32,
}
