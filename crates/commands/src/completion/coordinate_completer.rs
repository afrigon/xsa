use usage::spec::{Candidate, CompleteCtx};

use crate::value::{Latitude, Longitude};

pub(crate) fn complete_latitude<Partial>(_partial: &Partial, _context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    Latitude::NAMED
        .iter()
        .filter_map(|latitude| latitude.name())
        .map(Candidate::new)
        .collect()
}

pub(crate) fn complete_longitude<Partial>(_partial: &Partial, _context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    Longitude::names().map(Candidate::new).collect()
}
