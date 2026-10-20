use std::cmp::Ordering;
use std::time::Duration;

// How much keeping something in memory is worth, comparable across cached resources and streamed pages. Anything
// nothing uses goes before anything wanted. Of what nothing uses, the least reload time saved per byte goes first,
// weighed by how likely it is used again: the longer it went unused, the less likely. Of wanted pages, the least
// visible goes first.
#[derive(Clone, Copy, Debug)]
pub(crate) enum EvictionRank {
    Idle {
        // What bringing it back would take: reading, decoding and uploading what eviction frees.
        reload_time: Duration,
        bytes: u64,
        idle_updates: u64,
    },
    Wanted {
        // Its request score: the on-screen error while it is missing.
        score: f32,
    },
}

impl EvictionRank {
    // Least worth keeping first.
    pub fn eviction_order(a: &EvictionRank, b: &EvictionRank) -> Ordering {
        match (a, b) {
            (EvictionRank::Idle { .. }, EvictionRank::Wanted { .. }) => Ordering::Less,
            (EvictionRank::Wanted { .. }, EvictionRank::Idle { .. }) => Ordering::Greater,
            (EvictionRank::Wanted { score: a }, EvictionRank::Wanted { score: b }) => a.total_cmp(b),
            (
                EvictionRank::Idle {
                    idle_updates: a_idle, ..
                },
                EvictionRank::Idle {
                    idle_updates: b_idle, ..
                },
            ) => a.idle_value().total_cmp(&b.idle_value()).then(b_idle.cmp(a_idle)),
        }
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, EvictionRank::Idle { .. })
    }

    // The reload seconds it saves per byte for each update it stays, if used again.
    fn idle_value(&self) -> f64 {
        match *self {
            EvictionRank::Idle {
                reload_time,
                bytes,
                idle_updates,
            } => {
                if bytes == 0 {
                    return f64::INFINITY;
                }

                reload_time.as_secs_f64() / (bytes as f64 * (idle_updates + 1) as f64)
            }
            EvictionRank::Wanted { .. } => f64::INFINITY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIB: u64 = 1024 * 1024;

    fn idle(reload_millis: u64, bytes: u64, idle_updates: u64) -> EvictionRank {
        EvictionRank::Idle {
            reload_time: Duration::from_millis(reload_millis),
            bytes,
            idle_updates,
        }
    }

    fn first(a: EvictionRank, b: EvictionRank) -> bool {
        EvictionRank::eviction_order(&a, &b) == Ordering::Less
    }

    #[test]
    fn what_is_cheaper_to_reload_goes_first() {
        assert!(first(idle(10, 8 * MIB, 5), idle(1000, 8 * MIB, 5)));
    }

    #[test]
    fn of_two_as_costly_to_reload_the_larger_goes_first() {
        assert!(first(idle(100, 200 * MIB, 5), idle(100, 4 * MIB, 5)));
    }

    #[test]
    fn what_went_unused_longer_goes_first() {
        assert!(first(idle(100, 8 * MIB, 600), idle(100, 8 * MIB, 3)));
    }

    #[test]
    fn without_a_measured_cost_the_least_recently_used_goes_first() {
        assert!(first(idle(0, 8 * MIB, 9), idle(0, 8 * MIB, 2)));
    }

    #[test]
    fn anything_idle_goes_before_anything_wanted() {
        assert!(first(idle(60_000, MIB, 0), EvictionRank::Wanted { score: 0.0 }));
    }

    #[test]
    fn of_wanted_pages_the_least_visible_goes_first() {
        assert!(first(
            EvictionRank::Wanted { score: 1.0 },
            EvictionRank::Wanted { score: 8.0 }
        ));
    }
}
