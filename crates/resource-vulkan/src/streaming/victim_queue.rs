use std::collections::VecDeque;

use crate::streaming::Victim;

// The cells of one pool a page may take: unwanted pages least recently used first, then wanted pages least
// important first. A wanted page only gives way to a more important one, so loading in score order converges instead
// of thrashing. Pinned pages and frozen sets are never in it.
pub(crate) struct VictimQueue {
    victims: VecDeque<Victim>,
}

impl VictimQueue {
    pub fn new(mut victims: Vec<Victim>) -> VictimQueue {
        victims.sort_by(|a, b| match (a.score, b.score) {
            (None, None) => a.last_use.cmp(&b.last_use),
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => a.total_cmp(&b),
        });

        VictimQueue {
            victims: victims.into(),
        }
    }

    // The next page to give way to a page of `score`, if one matters less.
    pub fn take_for(&mut self, score: f32) -> Option<Victim> {
        let victim = self.victims.front()?;

        if victim.score.is_some_and(|victim_score| victim_score >= score) {
            return None;
        }

        self.victims.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::{PageId, PageOwner, SetId};

    fn victim(page: u64, last_use: u64, score: Option<f32>) -> Victim {
        Victim {
            owner: PageOwner {
                set: SetId { value: 1 },
                page: PageId { value: page },
            },
            last_use,
            score,
        }
    }

    fn page(victim: Option<Victim>) -> Option<u64> {
        victim.map(|victim| victim.owner.page.value)
    }

    #[test]
    fn unwanted_pages_go_first_least_recently_used_first() {
        let mut queue = VictimQueue::new(vec![victim(1, 9, Some(1.0)), victim(2, 5, None), victim(3, 2, None)]);

        assert_eq!(page(queue.take_for(0.5)), Some(3));
        assert_eq!(page(queue.take_for(0.5)), Some(2));
    }

    #[test]
    fn a_wanted_page_gives_way_only_to_a_more_important_one() {
        let mut queue = VictimQueue::new(vec![victim(1, 9, Some(4.0)), victim(2, 9, Some(2.0))]);

        assert_eq!(page(queue.take_for(2.0)), None);
        assert_eq!(page(queue.take_for(3.0)), Some(2));
        assert_eq!(page(queue.take_for(3.0)), None);
    }
}
