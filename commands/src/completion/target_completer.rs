use usage::spec::{Candidate, CompleteCtx};
use xsa_packs::NAMESPACE_SEPARATOR;

use super::{CompletionCandidate, CompletionKind};

// Body values are full ids; until a namespace is typed, offer short paths and the namespaces themselves.
// usage-rs calls completers as plain functions.
pub(crate) fn complete_target<Partial>(_partial: &Partial, context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    let bodies = CompletionCandidate::known(CompletionKind::Body);

    if context.prefix.contains(NAMESPACE_SEPARATOR) {
        return bodies.iter().map(CompletionCandidate::to_candidate).collect();
    }

    let mut candidates = vec![
        Candidate::described("next", "the next body"),
        Candidate::described("previous", "the previous body"),
    ];
    let mut namespaces = Vec::new();

    for body in &bodies {
        let Some((namespace, path)) = body.value.split_once(NAMESPACE_SEPARATOR) else {
            continue;
        };
        candidates.push(
            CompletionCandidate {
                value: path.to_string(),
                description: body.description.clone(),
                scope: None,
            }
            .to_candidate(),
        );

        let namespace = format!("{namespace}{NAMESPACE_SEPARATOR}");

        if !namespaces.contains(&namespace) {
            namespaces.push(namespace);
        }
    }

    candidates.extend(
        namespaces
            .into_iter()
            .map(|namespace| Candidate::described(namespace, "pack")),
    );

    candidates
}
