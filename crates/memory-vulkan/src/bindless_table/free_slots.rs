use crate::BindlessBinding;

// The slots of each binding no one holds: those never handed out, then those returned.
#[derive(Default)]
pub(crate) struct FreeSlots {
    next_unused: [u32; BindlessBinding::ALL.len()],
    returned: [Vec<u32>; BindlessBinding::ALL.len()],
}

impl FreeSlots {
    pub fn take(&mut self, binding: BindlessBinding) -> Option<u32> {
        if let Some(index) = self.returned[binding.index()].pop() {
            return Some(index);
        }

        let next = &mut self.next_unused[binding.index()];

        if *next == binding.capacity() {
            return None;
        }

        *next += 1;
        Some(*next - 1)
    }

    pub fn give_back(&mut self, binding: BindlessBinding, index: u32) {
        self.returned[binding.index()].push(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returned_slots_are_reused() {
        let mut slots = FreeSlots::default();
        let first = slots.take(BindlessBinding::Texture);
        slots.take(BindlessBinding::Texture);
        slots.give_back(BindlessBinding::Texture, 0);

        assert_eq!(first, Some(0));
        assert_eq!(slots.take(BindlessBinding::Texture), Some(0));
        assert_eq!(slots.take(BindlessBinding::Texture), Some(2));
    }

    #[test]
    fn bindings_count_separately() {
        let mut slots = FreeSlots::default();
        slots.take(BindlessBinding::Texture);

        assert_eq!(slots.take(BindlessBinding::CubeMap), Some(0));
    }

    #[test]
    fn a_full_binding_has_no_slot() {
        let mut slots = FreeSlots::default();

        for _ in 0..BindlessBinding::StorageImage.capacity() {
            slots.take(BindlessBinding::StorageImage);
        }

        assert_eq!(slots.take(BindlessBinding::StorageImage), None);
    }
}
