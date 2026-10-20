use std::collections::VecDeque;

use crate::EvictionRank;
use crate::streaming::Victim;

// The cells of one pool a page may take, least worth keeping first: unwanted pages before wanted ones, in the order
// the memory budget evicts. A wanted page only gives way to a more important one, so loading in score order converges
// instead of thrashing. Pinned pages, moving pages and frozen sets are never in it.
pub(crate) struct VictimQueue {
    victims: VecDeque<Victim>,
}

impl VictimQueue {
    pub fn new(mut victims: Vec<Victim>) -> VictimQueue {
        victims.sort_by(|a, b| EvictionRank::eviction_order(&a.rank, &b.rank));

        VictimQueue {
            victims: victims.into(),
        }
    }

    // The next page to give way to a page of `score`, if one matters less.
    pub fn take_for(&mut self, score: f32) -> Option<Victim> {
        let victim = self.victims.front()?;

        if let EvictionRank::Wanted { score: victim_score } = victim.rank
            && victim_score >= score
        {
            return None;
        }

        self.victims.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::streaming::{PageId, PageOwner, SetId};

    fn victim(page: u64, rank: EvictionRank) -> Victim {
        Victim {
            owner: PageOwner {
                set: SetId { value: 1 },
                page: PageId { value: page },
            },
            rank,
        }
    }

    fn idle(idle_updates: u64) -> EvictionRank {
        EvictionRank::Idle {
            reload_time: Duration::from_millis(5),
            bytes: 1024,
            idle_updates,
        }
    }

    fn wanted(score: f32) -> EvictionRank {
        EvictionRank::Wanted { score }
    }

    fn page(victim: Option<Victim>) -> Option<u64> {
        victim.map(|victim| victim.owner.page.value)
    }

    #[test]
    fn unwanted_pages_go_first_least_recently_used_first() {
        let mut queue = VictimQueue::new(vec![victim(1, wanted(1.0)), victim(2, idle(4)), victim(3, idle(7))]);

        assert_eq!(page(queue.take_for(0.5)), Some(3));
        assert_eq!(page(queue.take_for(0.5)), Some(2));
    }

    #[test]
    fn a_wanted_page_gives_way_only_to_a_more_important_one() {
        let mut queue = VictimQueue::new(vec![victim(1, wanted(4.0)), victim(2, wanted(2.0))]);

        assert_eq!(page(queue.take_for(2.0)), None);
        assert_eq!(page(queue.take_for(3.0)), Some(2));
        assert_eq!(page(queue.take_for(3.0)), None);
    }
}
