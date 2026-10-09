use usage::spec::{Candidate, CompleteCtx};

use super::{CompletionCandidate, CompletionKind};

const CONFIG_KEY_SEPARATOR: char = '.';

// Keys complete one segment at a time; typing the separator moves on to the next one.
pub(crate) fn complete_config_key<Partial>(_partial: &Partial, context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    let segment_start = context.prefix.rfind(CONFIG_KEY_SEPARATOR).map_or(0, |index| index + 1);
    let mut segments: Vec<String> = Vec::new();

    for key in CompletionCandidate::known(CompletionKind::ConfigKey) {
        if !key.value.starts_with(context.prefix) {
            continue;
        }

        let segment_end = key.value[segment_start..]
            .find(CONFIG_KEY_SEPARATOR)
            .map_or(key.value.len(), |index| segment_start + index);
        let segment = &key.value[..segment_end];

        if !segments.iter().any(|known| known == segment) {
            segments.push(segment.to_string());
        }
    }

    segments
        .into_iter()
        .map(|segment| {
            let display = segment[segment_start..].to_string();

            Candidate::new(segment).displayed(display)
        })
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
