use crate::Timeline;

/// A point on a timeline: the value a submission signals once it completes.
#[derive(Clone)]
pub struct TimelineValue {
    pub timeline: Timeline,
    pub value: u64,
}
