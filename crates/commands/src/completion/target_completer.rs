use usage::spec::{Candidate, CompleteCtx};
use xsa_packs::NAMESPACE_SEPARATOR;

use super::complete_body;

// usage-rs calls completers as plain functions.
pub(crate) fn complete_target<Partial>(partial: &Partial, context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    let mut candidates = if context.prefix.contains(NAMESPACE_SEPARATOR) {
        Vec::new()
    } else {
        vec![
            Candidate::described("next", "the next body"),
            Candidate::described("previous", "the previous body"),
            Candidate::described("parent", "the body the target orbits"),
        ]
    };
    candidates.extend(complete_body(partial, context));

    candidates
}
