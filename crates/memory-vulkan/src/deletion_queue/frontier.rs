/// How far GPU work has progressed: a value per timeline, indexed by timeline, and a count of presents.
///
/// As the work submitted so far, it is the point an entry of the deletion queue waits for; as the work completed so
/// far, it is what that point is compared against.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Frontier {
    pub timelines: Vec<u64>,
    pub presents: u64,
}

impl Frontier {
    /// Whether every piece of work up to this frontier is part of `completed`.
    pub fn reached_by(&self, completed: &Frontier) -> bool {
        let timelines_reached = self
            .timelines
            .iter()
            .enumerate()
            .all(|(index, &value)| completed.timelines.get(index).copied().unwrap_or(0) >= value);

        timelines_reached && completed.presents >= self.presents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frontier(timelines: &[u64], presents: u64) -> Frontier {
        Frontier {
            timelines: timelines.to_vec(),
            presents,
        }
    }

    #[test]
    fn reached_when_every_timeline_and_present_is_reached() {
        assert!(frontier(&[3, 1], 2).reached_by(&frontier(&[3, 4], 2)));
    }

    #[test]
    fn not_reached_when_one_timeline_lags() {
        assert!(!frontier(&[3, 5], 0).reached_by(&frontier(&[9, 4], 0)));
    }

    #[test]
    fn not_reached_when_presents_lag() {
        assert!(!frontier(&[1], 2).reached_by(&frontier(&[1], 1)));
    }

    #[test]
    fn a_timeline_created_later_counts_as_nothing_completed() {
        assert!(!frontier(&[1, 1], 0).reached_by(&frontier(&[1], 0)));
        assert!(frontier(&[1, 0], 0).reached_by(&frontier(&[1], 0)));
    }
}
