use std::cell::RefCell;
use std::collections::HashMap;

use bitcode::{Decode, Encode};
use tokio::sync::oneshot;
use usage::complete::{self, Shell};
#[cfg(feature = "client")]
use usage::spec::{Candidate, CompleteCtx};
#[cfg(feature = "client")]
use xsa_core::packs::id::NAMESPACE_SEPARATOR;

use crate::command::CommandLine;

// Completion splits a full command line, whose first word is the program.
const PROGRAM_WORD: &str = "xsa ";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionKind {
    Body,
}

const COMPLETION_KINDS: [CompletionKind; 1] = [CompletionKind::Body];

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidate {
    pub value: String,
    pub description: Option<String>,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, Default)]
pub struct Completions {
    pub start: usize,
    pub candidates: Vec<CompletionCandidate>,
}

pub struct CommandCompletion {
    pub line: String,
    pub cursor: usize,
    pub reply: oneshot::Sender<Completions>,
}

thread_local! {
    // usage-rs completers are plain functions; the executor's values reach them through here for one call.
    static VALUES: RefCell<HashMap<CompletionKind, Vec<CompletionCandidate>>> = RefCell::new(HashMap::new());
}

pub fn complete(line: &str, cursor: usize, values: impl Fn(CompletionKind) -> Vec<CompletionCandidate>) -> Completions {
    let cursor = cursor.min(line.len());
    let full_line = format!("{PROGRAM_WORD}{line}");
    let split = complete::split(&full_line, PROGRAM_WORD.len() + cursor, Shell::Bash);
    VALUES.with_borrow_mut(|known| {
        known.clear();
        known.extend(COMPLETION_KINDS.map(|kind| (kind, values(kind))));
    });
    let candidates = complete::candidates(CommandLine::spec(), &split)
        .into_iter()
        .filter(|candidate| candidate.value.starts_with(&split.prefix))
        .map(|candidate| CompletionCandidate {
            value: candidate.value,
            description: candidate.description.map(|description| description.into_owned()),
        })
        .collect();
    VALUES.with_borrow_mut(HashMap::clear);
    Completions {
        start: cursor.saturating_sub(split.prefix.len()),
        candidates,
    }
}

// Body values are full ids; until a namespace is typed, offer short paths and the namespaces themselves.
#[cfg(feature = "client")]
pub(crate) fn complete_target<Partial>(_partial: &Partial, context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    let bodies = known(CompletionKind::Body);
    if context.prefix.contains(NAMESPACE_SEPARATOR) {
        return bodies.iter().map(candidate).collect();
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
        candidates.push(candidate(&CompletionCandidate {
            value: path.to_string(),
            description: body.description.clone(),
        }));
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

#[cfg(feature = "client")]
fn known(kind: CompletionKind) -> Vec<CompletionCandidate> {
    VALUES.with_borrow(|known| known.get(&kind).cloned().unwrap_or_default())
}

#[cfg(feature = "client")]
fn candidate(value: &CompletionCandidate) -> Candidate<'static> {
    match &value.description {
        Some(description) => Candidate::described(value.value.clone(), description.clone()),
        None => Candidate::new(value.value.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(text: &[&str]) -> impl Fn(CompletionKind) -> Vec<CompletionCandidate> {
        let values: Vec<CompletionCandidate> = text
            .iter()
            .map(|value| CompletionCandidate {
                value: value.to_string(),
                description: None,
            })
            .collect();
        move |_| values.clone()
    }

    fn completed(line: &str, known: &[&str]) -> Vec<String> {
        complete(line, line.len(), values(known))
            .candidates
            .into_iter()
            .map(|candidate| candidate.value)
            .collect()
    }

    #[test]
    fn commands_complete_from_the_tree() {
        assert!(completed("ti", &[]).contains(&"time".to_string()));
        assert!(completed("time ", &[]).contains(&"rate".to_string()));
    }

    #[cfg(feature = "client")]
    #[test]
    fn targets_complete_from_the_executor_values() {
        let bodies = ["system-solar:mars", "system-solar:mercury", "system-solar:luna"];
        assert_eq!(completed("camera target ma", &bodies), ["mars"]);
        assert_eq!(completed("camera target syst", &bodies), ["system-solar:"]);
        assert_eq!(
            completed("camera target system-solar:lu", &bodies),
            ["system-solar:luna"]
        );
        let all = completed("camera look-at ", &bodies);
        for value in ["next", "previous", "luna", "system-solar:"] {
            assert!(all.contains(&value.to_string()), "{value} in {all:?}");
        }
    }

    #[test]
    fn the_start_is_where_the_completed_word_begins() {
        assert_eq!(complete("time ra", 7, values(&[])).start, 5);
    }
}
