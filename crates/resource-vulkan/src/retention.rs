/// What happens to a resource once its last handle is dropped.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Retention {
    /// Kept until memory is needed, so loading it again is free.
    #[default]
    Cache,
    /// Freed at once: loading-screen art and other one-time content.
    Discard,
}
