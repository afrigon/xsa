mod budget_reading;

pub(crate) use budget_reading::BudgetReading;

use crate::{MemoryRoom, MemoryUsage, ResourceLimits};

// Decides how far usage is over budget and how much room is left. Resources and streamed pages fill memory up to what
// the system reports; the limits only lower that.
#[derive(Default)]
pub(crate) struct MemoryBudget {
    pub limits: ResourceLimits,
}

impl MemoryBudget {
    // The bytes of video memory and RAM above budget.
    pub fn overage(&self, reading: &BudgetReading) -> MemoryUsage {
        let ram = self
            .ram_budget(reading)
            .map_or(0, |budget| reading.held_ram.saturating_sub(budget));

        MemoryUsage {
            video: self.video_in_use(reading).saturating_sub(self.video_budget(reading)),
            ram,
        }
    }

    // What can be taken without going over budget.
    pub fn room(&self, reading: &BudgetReading) -> MemoryRoom {
        MemoryRoom {
            video: self.video_budget(reading).saturating_sub(self.video_in_use(reading)),
            ram: self
                .ram_budget(reading)
                .map(|budget| budget.saturating_sub(reading.held_ram)),
        }
    }

    fn video_budget(&self, reading: &BudgetReading) -> u64 {
        MemoryBudget::capped(reading.video.budget, self.limits.max_video_memory_usage)
    }

    // What objects occupy: the driver counts whole allocator blocks, though freed space in them is free for new
    // objects.
    fn video_in_use(&self, reading: &BudgetReading) -> u64 {
        reading
            .video
            .usage
            .saturating_sub(reading.video.unallocated)
            .saturating_sub(reading.evicting_video)
    }

    fn ram_budget(&self, reading: &BudgetReading) -> Option<u64> {
        let reported = reading.available_ram.map(|available| reading.held_ram + available);

        match (reported, self.limits.max_memory_usage) {
            (Some(reported), limit) => Some(MemoryBudget::capped(reported, limit)),
            (None, limit) => limit,
        }
    }

    fn capped(reported: u64, limit: Option<u64>) -> u64 {
        limit.map_or(reported, |limit| reported.min(limit))
    }
}

#[cfg(test)]
mod tests {
    use memory_vulkan::HeapUsage;

    use super::*;

    const MIB: u64 = 1024 * 1024;

    fn reading(video_budget: u64, video_usage: u64) -> BudgetReading {
        BudgetReading {
            video: HeapUsage {
                budget: video_budget,
                usage: video_usage,
                unallocated: 0,
            },
            evicting_video: 0,
            held_ram: 0,
            available_ram: Some(1024 * MIB),
        }
    }

    fn video_over(budget: &MemoryBudget, reading: &BudgetReading) -> u64 {
        budget.overage(reading).video
    }

    #[test]
    fn nothing_is_over_within_budget() {
        assert_eq!(video_over(&MemoryBudget::default(), &reading(1000 * MIB, 900 * MIB)), 0);
    }

    #[test]
    fn a_limit_lowers_the_reported_budget() {
        let budget = MemoryBudget {
            limits: ResourceLimits {
                max_video_memory_usage: Some(500 * MIB),
                ..ResourceLimits::default()
            },
        };

        assert_eq!(video_over(&budget, &reading(1000 * MIB, 600 * MIB)), 100 * MIB);
    }

    #[test]
    fn memory_already_evicting_counts_as_freed() {
        let mut over = reading(1000 * MIB, 1100 * MIB);
        over.evicting_video = 100 * MIB;

        assert_eq!(video_over(&MemoryBudget::default(), &over), 0);
    }

    #[test]
    fn space_no_object_occupies_in_allocator_blocks_counts_as_free() {
        let mut over = reading(1000 * MIB, 1100 * MIB);
        over.video.unallocated = 150 * MIB;

        assert_eq!(video_over(&MemoryBudget::default(), &over), 0);
        assert_eq!(MemoryBudget::default().room(&over).video, 50 * MIB);
    }

    #[test]
    fn a_ram_limit_puts_ram_over() {
        let budget = MemoryBudget {
            limits: ResourceLimits {
                max_memory_usage: Some(100 * MIB),
                ..ResourceLimits::default()
            },
        };
        let mut over = reading(1000 * MIB, 0);
        over.held_ram = 150 * MIB;

        assert_eq!(budget.overage(&over).ram, 50 * MIB);
        assert_eq!(budget.room(&over).ram, Some(0));
    }

    #[test]
    fn unreadable_ram_without_a_limit_has_no_budget() {
        let mut over = reading(1000 * MIB, 0);
        over.available_ram = None;
        over.held_ram = 150 * MIB;

        assert_eq!(MemoryBudget::default().overage(&over).ram, 0);
        assert_eq!(MemoryBudget::default().room(&over).ram, None);
    }
}
