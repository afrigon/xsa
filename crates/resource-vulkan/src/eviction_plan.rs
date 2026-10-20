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
