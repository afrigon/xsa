use usage::spec::{Candidate, CompleteCtx};
use xsa_core::simulation::BodyCategory;

pub(crate) fn complete_body_category<Partial>(
    _partial: &Partial,
    _context: &CompleteCtx<'_>,
) -> Vec<Candidate<'static>> {
    BodyCategory::ALL
        .iter()
        .map(|category| Candidate::new(category.name()))
        .collect()
}
