use std::sync::{Arc, Mutex};

use crate::bindless_table::FreeSlots;
use crate::{BindlessBinding, MemoryContext};

/// One slot of the bindless table, held by whoever owns the image written into it. Dropping it returns the slot once
/// the GPU has finished every frame that may read it.
pub struct BindlessSlot {
    binding: BindlessBinding,
    index: u32,
    free: Arc<Mutex<FreeSlots>>,
    memory: MemoryContext,
}

impl BindlessSlot {
    pub(crate) fn new(
        binding: BindlessBinding,
        index: u32,
        free: Arc<Mutex<FreeSlots>>,
        memory: MemoryContext,
    ) -> BindlessSlot {
        BindlessSlot {
            binding,
            index,
            free,
            memory,
        }
    }

    pub fn binding(&self) -> BindlessBinding {
        self.binding
    }

    /// The index shaders read the slot's image at, in its binding's array.
    pub fn index(&self) -> u32 {
        self.index
    }
}

impl Drop for BindlessSlot {
    fn drop(&mut self) {
        let free = Arc::clone(&self.free);
        let binding = self.binding;
        let index = self.index;
        self.memory.defer_action(move |_| {
            free.lock()
                .expect("the bindless free slots' lock is poisoned")
                .give_back(binding, index);
        });
    }
}
