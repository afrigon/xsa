use crate::streaming::PoolCapacity;
use crate::{EvictionPlan, MemoryRoom, MemoryUsage, SoftUsage};

// Decides what may load in one update. Something loads only if it fits in the room within budget plus what evicting
// idle memory would free, so making room never evicts anything wanted: what eviction takes from wanted pages is not
// loaded again until room appears, and loading and evicting cannot chase each other. What it admits is taken from
// the room, so the same memory is not promised twice.
pub(crate) struct Admission {
    room: MemoryRoom,
    idle_entries: Vec<SoftUsage>,
    // Counted the first time the room alone is not enough.
    reclaimable: Option<MemoryUsage>,
}

impl Admission {
    pub fn new(room: MemoryRoom, idle_entries: Vec<SoftUsage>) -> Admission {
        Admission {
            room,
            idle_entries,
            reclaimable: None,
        }
    }

    // Takes `need` from the room, and from idle memory past it; `idle` lists the streamed pages' idle usage and their
    // pools. False, taking nothing, when they cannot cover it.
    pub fn admit(&mut self, need: MemoryUsage, idle: impl FnOnce() -> (Vec<SoftUsage>, Vec<PoolCapacity>)) -> bool {
        let ram_room = self.room.ram.unwrap_or(u64::MAX);

        if need.video <= self.room.video && need.ram <= ram_room {
            self.take(need, MemoryUsage::default());
            return true;
        }

        let reclaimable = *self.reclaimable.get_or_insert_with(|| {
            let (mut soft, pools) = idle();
            soft.extend(self.idle_entries.iter().copied());

            EvictionPlan::reclaimable(&soft, &pools)
        });
        let fits_video = need.video <= self.room.video.saturating_add(reclaimable.video);
        let fits_ram = need.ram <= ram_room.saturating_add(reclaimable.ram);

        if !fits_video || !fits_ram {
            return false;
        }

        let from_reclaimable = MemoryUsage {
            video: need.video.saturating_sub(self.room.video),
            ram: need.ram.saturating_sub(ram_room),
        };
        self.take(need, from_reclaimable);

        true
    }

    fn take(&mut self, need: MemoryUsage, from_reclaimable: MemoryUsage) {
        self.room.video = self.room.video.saturating_sub(need.video);
        self.room.ram = self.room.ram.map(|ram| ram.saturating_sub(need.ram));

        if let Some(reclaimable) = &mut self.reclaimable {
            reclaimable.video -= from_reclaimable.video;
            reclaimable.ram -= from_reclaimable.ram;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::{EntryId, EvictionRank, SoftOwner};

    const MIB: u64 = 1024 * 1024;

    fn entry(video: u64, rank: EvictionRank) -> SoftUsage {
        SoftUsage {
            owner: SoftOwner::Entry(EntryId { value: 1 }),
            memory: MemoryUsage { video, ram: 0 },
            rank,
        }
    }

    fn idle_rank() -> EvictionRank {
        EvictionRank::Idle {
            reload_time: Duration::from_millis(10),
            bytes: MIB,
            idle_updates: 1,
        }
    }

    fn video(bytes: u64) -> MemoryUsage {
        MemoryUsage { video: bytes, ram: 0 }
    }

    fn no_pages() -> (Vec<SoftUsage>, Vec<PoolCapacity>) {
        (Vec::new(), Vec::new())
    }

    #[test]
    fn what_fits_in_the_room_is_admitted_until_the_room_is_taken() {
        let room = MemoryRoom {
            video: 64 * MIB,
            ram: None,
        };
        let mut admission = Admission::new(room, Vec::new());

        assert!(admission.admit(video(48 * MIB), no_pages));
        assert!(!admission.admit(video(32 * MIB), no_pages));
    }

    #[test]
    fn idle_memory_makes_room_once() {
        let mut admission = Admission::new(MemoryRoom::default(), vec![entry(32 * MIB, idle_rank())]);

        assert!(admission.admit(video(32 * MIB), no_pages));
        assert!(!admission.admit(video(1), no_pages));
    }

    #[test]
    fn ram_without_a_budget_is_always_admitted() {
        let mut admission = Admission::new(MemoryRoom::default(), Vec::new());

        assert!(admission.admit(
            MemoryUsage {
                video: 0,
                ram: u64::MAX
            },
            no_pages
        ));
    }
}
