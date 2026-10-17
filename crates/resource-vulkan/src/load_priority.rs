use memory_vulkan::TransferPriority;

/// How urgently a resource is needed: `Background` prefetches, `Critical` is needed for the next frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LoadPriority {
    Background,
    Normal,
    High,
    Critical,
}

impl LoadPriority {
    pub(crate) const ALL: [LoadPriority; 4] = [
        LoadPriority::Background,
        LoadPriority::Normal,
        LoadPriority::High,
        LoadPriority::Critical,
    ];

    pub fn transfer(self) -> TransferPriority {
        match self {
            LoadPriority::Background => TransferPriority::Background,
            LoadPriority::Normal => TransferPriority::Normal,
            LoadPriority::High => TransferPriority::High,
            LoadPriority::Critical => TransferPriority::Critical,
        }
    }

    pub(crate) fn index(self) -> usize {
        self as usize
    }

    pub(crate) fn from_index(index: usize) -> LoadPriority {
        LoadPriority::ALL[index]
    }
}
