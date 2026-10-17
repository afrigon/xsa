use ash::vk;

// Where copies run: a family of their own lets the copy engine work in parallel with rendering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferQueueLocation {
    DedicatedFamily(u32),
    // A second queue of the graphics family.
    GraphicsFamily,
    // The graphics queue itself, sharing its timeline.
    GraphicsQueue,
}

impl TransferQueueLocation {
    pub fn select(queue_families: &[vk::QueueFamilyProperties], graphics_family: u32) -> TransferQueueLocation {
        let dedicated = (0..).zip(queue_families).find(|(_, family)| {
            family.queue_flags.contains(vk::QueueFlags::TRANSFER)
                && !family
                    .queue_flags
                    .intersects(vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE)
        });

        match dedicated {
            Some((index, _)) => TransferQueueLocation::DedicatedFamily(index),
            None if queue_families[graphics_family as usize].queue_count > 1 => TransferQueueLocation::GraphicsFamily,
            None => TransferQueueLocation::GraphicsQueue,
        }
    }

    pub fn family(self, graphics_family: u32) -> u32 {
        match self {
            TransferQueueLocation::DedicatedFamily(family) => family,
            TransferQueueLocation::GraphicsFamily | TransferQueueLocation::GraphicsQueue => graphics_family,
        }
    }
}
