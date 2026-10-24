use usage::spec::{Candidate, CompleteCtx};
use xsa_core::daylight::TimeOfDay;

pub(crate) fn complete_time_of_day<Partial>(_partial: &Partial, _context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    TimeOfDay::ALL
        .iter()
        .map(|time_of_day| Candidate::new(time_of_day.name()))
        .collect()
}
