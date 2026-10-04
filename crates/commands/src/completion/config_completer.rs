use usage::spec::{Candidate, CompleteCtx};

use super::{CompletionCandidate, CompletionKind};

pub(crate) fn complete_config_key<Partial>(_partial: &Partial, _context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    CompletionCandidate::known(CompletionKind::ConfigKey)
        .iter()
        .map(CompletionCandidate::to_candidate)
        .collect()
}

// The value's choices depend on the key typed before it.
pub(crate) fn complete_config_value<Partial>(_partial: &Partial, context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    let Some(key) = context.command_words.iter().find(|word| !word.starts_with('-')) else {
        return Vec::new();
    };

    CompletionCandidate::known(CompletionKind::ConfigValue)
        .iter()
        .filter(|value| value.scope.as_deref() == Some(key.as_str()))
        .map(CompletionCandidate::to_candidate)
        .collect()
}
