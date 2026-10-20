use std::collections::HashMap;

use crate::streaming::{PageFormat, PageOwner, PoolCapacity, PoolShrink};
use crate::{EntryId, MemoryUsage, SoftOwner, SoftUsage};

// What to evict to cover an overage, picked in the order of the soft usage given. Streamed pages free video memory a
// chunk at a time: a pool releases a chunk once its free cells and the cells of the pages picked from it fill one,
// so pages picked past its last whole chunk stay. Pools missing from the capacities given are not shrunk.
#[derive(Debug, Default)]
pub(crate) struct EvictionPlan {
    pub entries: Vec<EntryId>,
    pub page_bytes: Vec<PageOwner>,
    pub shrinks: Vec<PoolShrink>,
    pub freed: MemoryUsage,
}

// The pages picked from one pool so far.
struct PoolPick {
    capacity: PoolCapacity,
    picked: Vec<PageOwner>,
    // Picked pages that complete a whole chunk.
    committed: usize,
    chunks: usize,
}

impl EvictionPlan {
    pub fn new(overage: MemoryUsage, soft: &[SoftUsage], pools: &[PoolCapacity]) -> EvictionPlan {
        let mut plan = EvictionPlan::default();
        let mut over = overage;
        let mut picks: HashMap<PageFormat, PoolPick> = pools
            .iter()
            .map(|&capacity| {
                (
                    capacity.format,
                    PoolPick {
                        capacity,
                        picked: Vec::new(),
                        committed: 0,
                        chunks: 0,
                    },
                )
            })
            .collect();

        for pick in picks.values_mut() {
            while over.video > 0 && pick.fills_another_chunk() {
                plan.free_chunk(pick, &mut over);
            }
        }

        for usage in soft {
            if over.video == 0 && over.ram == 0 {
                break;
            }

            match usage.owner {
                SoftOwner::Entry(entry) => {
                    if (over.video > 0 && usage.memory.video > 0) || (over.ram > 0 && usage.memory.ram > 0) {
                        plan.entries.push(entry);
                        plan.free(usage.memory, &mut over);
                    }
                }
                SoftOwner::PageBytes(page) => {
                    if over.ram > 0 && usage.memory.ram > 0 {
                        plan.page_bytes.push(page);
                        plan.free(usage.memory, &mut over);
                    }
                }
                SoftOwner::Cell { pool, page } => {
                    let Some(pick) = picks.get_mut(&pool).filter(|_| over.video > 0) else {
                        continue;
                    };
                    pick.picked.push(page);

                    if pick.fills_another_chunk() {
                        pick.committed = pick.picked.len();
                        plan.free_chunk(pick, &mut over);
                    }
                }
            }
        }

        plan.shrinks = picks
            .into_values()
            .filter(|pick| pick.chunks > 0)
            .map(|mut pick| {
                pick.picked.truncate(pick.committed);

                PoolShrink {
                    format: pick.capacity.format,
                    evicted: pick.picked,
                    chunks: pick.chunks,
                }
            })
            .collect();

        plan
    }

    // Everything `soft` could free, in the units eviction frees it in.
    pub fn reclaimable(soft: &[SoftUsage], pools: &[PoolCapacity]) -> MemoryUsage {
        let unlimited = MemoryUsage {
            video: u64::MAX,
            ram: u64::MAX,
        };

        EvictionPlan::new(unlimited, soft, pools).freed
    }

    fn free(&mut self, memory: MemoryUsage, over: &mut MemoryUsage) {
        self.freed += memory;
        over.video = over.video.saturating_sub(memory.video);
        over.ram = over.ram.saturating_sub(memory.ram);
    }

    fn free_chunk(&mut self, pick: &mut PoolPick, over: &mut MemoryUsage) {
        pick.chunks += 1;
        self.free(
            MemoryUsage {
                video: pick.capacity.chunk_size,
                ram: 0,
            },
            over,
        );
    }
}

impl PoolPick {
    fn fills_another_chunk(&self) -> bool {
        let cells = self.capacity.free_cells + self.picked.len();

        cells >= self.capacity.cells_per_chunk * (self.chunks + 1)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ash::vk;
    use memory_vulkan::TexelBlock;

    use super::*;
    use crate::EvictionRank;
    use crate::streaming::{PageId, SetId};

    const MIB: u64 = 1024 * 1024;
    const CELLS: usize = 4;
    const CHUNK: u64 = 32 * MIB;

    fn format() -> PageFormat {
        PageFormat {
            format: vk::Format::R8G8B8A8_UNORM,
            extent: vk::Extent2D { width: 64, height: 64 },
            texel_block: TexelBlock {
                extent: vk::Extent2D { width: 1, height: 1 },
                size: 4,
            },
        }
    }

    fn pool(free_cells: usize) -> PoolCapacity {
        PoolCapacity {
            format: format(),
            cells_per_chunk: CELLS,
            chunk_size: CHUNK,
            free_cells,
        }
    }

    fn idle() -> EvictionRank {
        EvictionRank::Idle {
            reload_time: Duration::from_millis(1),
            bytes: MIB,
            idle_updates: 1,
        }
    }

    fn page(value: u64) -> PageOwner {
        PageOwner {
            set: SetId { value: 1 },
            page: PageId { value },
        }
    }

    fn cell(value: u64) -> SoftUsage {
        SoftUsage {
            owner: SoftOwner::Cell {
                pool: format(),
                page: page(value),
            },
            memory: MemoryUsage {
                video: CHUNK / CELLS as u64,
                ram: 0,
            },
            rank: idle(),
        }
    }

    fn entry(value: u64, video: u64) -> SoftUsage {
        SoftUsage {
            owner: SoftOwner::Entry(EntryId { value }),
            memory: MemoryUsage { video, ram: 0 },
            rank: idle(),
        }
    }

    fn video(bytes: u64) -> MemoryUsage {
        MemoryUsage { video: bytes, ram: 0 }
    }

    #[test]
    fn entries_go_in_order_until_the_overage_is_covered() {
        let soft = [entry(1, 60 * MIB), entry(2, 60 * MIB), entry(3, 60 * MIB)];
        let plan = EvictionPlan::new(video(100 * MIB), &soft, &[]);

        assert_eq!(plan.entries, vec![EntryId { value: 1 }, EntryId { value: 2 }]);
    }

    #[test]
    fn pages_free_video_memory_only_a_whole_chunk_at_a_time() {
        let soft = [cell(1), cell(2), entry(3, 10 * MIB), cell(4), cell(5)];
        let plan = EvictionPlan::new(video(20 * MIB), &soft, &[pool(0)]);

        assert_eq!(plan.entries, vec![EntryId { value: 3 }]);
        assert_eq!(plan.shrinks.len(), 1);
        assert_eq!(plan.shrinks[0].evicted, vec![page(1), page(2), page(4), page(5)]);
        assert_eq!(plan.freed.video, 10 * MIB + CHUNK);
    }

    #[test]
    fn pages_past_the_last_whole_chunk_stay() {
        let soft = [cell(1), cell(2), entry(3, 40 * MIB)];
        let plan = EvictionPlan::new(video(40 * MIB), &soft, &[pool(0)]);

        assert!(plan.shrinks.is_empty());
        assert_eq!(plan.entries, vec![EntryId { value: 3 }]);
    }

    #[test]
    fn free_cells_fill_a_chunk_before_any_page_goes() {
        let plan = EvictionPlan::new(video(CHUNK), &[cell(1), cell(2)], &[pool(3)]);

        assert_eq!(plan.shrinks[0].evicted, vec![page(1)]);
        assert_eq!(plan.shrinks[0].chunks, 1);
    }

    #[test]
    fn a_pool_not_given_is_not_shrunk() {
        let plan = EvictionPlan::new(video(CHUNK), &[cell(1), cell(2), cell(3), cell(4)], &[]);

        assert!(plan.shrinks.is_empty());
        assert_eq!(plan.freed, MemoryUsage::default());
    }

    #[test]
    fn reclaimable_memory_counts_pages_in_whole_chunks_only() {
        let soft = [cell(1), cell(2), cell(3), cell(4), cell(5), entry(6, MIB)];

        assert_eq!(EvictionPlan::reclaimable(&soft, &[pool(1)]).video, CHUNK + MIB);
    }
}
