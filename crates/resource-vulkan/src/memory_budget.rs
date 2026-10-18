mod budget_reading;

pub(crate) use budget_reading::BudgetReading;

use crate::{MemoryUsage, ResourceLimits};

// Decides how far usage is over budget and which soft usage goes. Resources and streamed pages fill memory up to what
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

    // The video memory that can be taken without going over budget once `reclaimable` bytes of soft usage go.
    pub fn video_room(&self, reading: &BudgetReading, reclaimable: u64) -> u64 {
        let in_use = self.video_in_use(reading).saturating_sub(reclaimable);

        self.video_budget(reading).saturating_sub(in_use)
    }

    // `soft` lists what may go, in the order it should; picks from it until `overage` is covered.
    pub fn evictions<T>(overage: MemoryUsage, soft: impl IntoIterator<Item = (T, MemoryUsage)>) -> Vec<T> {
        let mut video_over = overage.video;
        let mut ram_over = overage.ram;
        let mut evicted = Vec::new();

        for (owner, memory) in soft {
            if video_over == 0 && ram_over == 0 {
                break;
            }

            let frees_video = video_over > 0 && memory.video > 0;
            let frees_ram = ram_over > 0 && memory.ram > 0;

            if frees_video || frees_ram {
                video_over = video_over.saturating_sub(memory.video);
                ram_over = ram_over.saturating_sub(memory.ram);
                evicted.push(owner);
            }
        }

        evicted
    }

    fn video_budget(&self, reading: &BudgetReading) -> u64 {
        MemoryBudget::capped(reading.video.budget, self.limits.max_video_memory_usage)
    }

    fn video_in_use(&self, reading: &BudgetReading) -> u64 {
        reading.video.usage.saturating_sub(reading.evicting_video)
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

    fn entry(value: u64, video: u64, ram: u64) -> (u64, MemoryUsage) {
        (value, MemoryUsage { video, ram })
    }

    fn reading(video_budget: u64, video_usage: u64) -> BudgetReading {
        BudgetReading {
            video: HeapUsage {
                budget: video_budget,
                usage: video_usage,
            },
            evicting_video: 0,
            held_ram: 0,
            available_ram: Some(1024 * MIB),
        }
    }

    fn evicted(budget: &MemoryBudget, reading: &BudgetReading, soft: Vec<(u64, MemoryUsage)>) -> Vec<u64> {
        MemoryBudget::evictions(budget.overage(reading), soft)
    }

    #[test]
    fn nothing_goes_within_budget() {
        let soft = vec![entry(0, 100 * MIB, 0)];

        assert!(evicted(&MemoryBudget::default(), &reading(1000 * MIB, 900 * MIB), soft).is_empty());
    }

    #[test]
    fn least_recently_used_go_first_until_within_budget() {
        let soft = vec![entry(0, 60 * MIB, 0), entry(1, 60 * MIB, 0), entry(2, 60 * MIB, 0)];

        assert_eq!(
            evicted(&MemoryBudget::default(), &reading(1000 * MIB, 1100 * MIB), soft),
            vec![0, 1]
        );
    }

    #[test]
    fn a_limit_lowers_the_reported_budget() {
        let budget = MemoryBudget {
            limits: ResourceLimits {
                max_video_memory_usage: Some(500 * MIB),
                ..ResourceLimits::default()
            },
        };
        let soft = vec![entry(0, 100 * MIB, 0), entry(1, 100 * MIB, 0)];

        assert_eq!(evicted(&budget, &reading(1000 * MIB, 600 * MIB), soft), vec![0]);
    }

    #[test]
    fn memory_already_evicting_counts_as_freed() {
        let mut over = reading(1000 * MIB, 1100 * MIB);
        over.evicting_video = 100 * MIB;

        assert!(evicted(&MemoryBudget::default(), &over, vec![entry(0, 100 * MIB, 0)]).is_empty());
    }

    #[test]
    fn only_resources_holding_the_scarce_memory_go() {
        let soft = vec![entry(0, 0, 50 * MIB), entry(1, 100 * MIB, 0)];

        assert_eq!(
            evicted(&MemoryBudget::default(), &reading(1000 * MIB, 1050 * MIB), soft),
            vec![1]
        );
    }

    #[test]
    fn a_ram_limit_evicts_ram() {
        let budget = MemoryBudget {
            limits: ResourceLimits {
                max_memory_usage: Some(100 * MIB),
                ..ResourceLimits::default()
            },
        };
        let mut over = reading(1000 * MIB, 0);
        over.held_ram = 150 * MIB;
        let soft = vec![entry(0, 10 * MIB, 0), entry(1, 0, 80 * MIB)];

        assert_eq!(evicted(&budget, &over, soft), vec![1]);
    }

    #[test]
    fn unreadable_ram_is_kept_within_its_limit_only() {
        let mut over = reading(1000 * MIB, 0);
        over.available_ram = None;
        over.held_ram = 150 * MIB;

        assert!(evicted(&MemoryBudget::default(), &over, vec![entry(0, 0, 80 * MIB)]).is_empty());
    }

    #[test]
    fn room_counts_reclaimable_usage_as_free() {
        let budget = MemoryBudget::default();
        let full = reading(1000 * MIB, 1000 * MIB);

        assert_eq!(budget.video_room(&full, 0), 0);
        assert_eq!(budget.video_room(&full, 64 * MIB), 64 * MIB);
    }
}
