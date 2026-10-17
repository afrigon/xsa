use ash::vk;

/// Which queue families may use a buffer or image. Concurrent sharing lets several families use it without ownership
/// transfers; it needs at least two distinct families.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueueSharing {
    Exclusive,
    Concurrent(Vec<u32>),
}

impl QueueSharing {
    pub(crate) fn mode(&self) -> vk::SharingMode {
        match self {
            QueueSharing::Exclusive => vk::SharingMode::EXCLUSIVE,
            QueueSharing::Concurrent(_) => vk::SharingMode::CONCURRENT,
        }
    }

    pub(crate) fn families(&self) -> &[u32] {
        match self {
            QueueSharing::Exclusive => &[],
            QueueSharing::Concurrent(families) => families,
        }
    }
}
