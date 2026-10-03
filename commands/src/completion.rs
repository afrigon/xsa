use std::cell::RefCell;
use std::collections::HashMap;

use bitcode::{Decode, Encode};
use tokio::sync::oneshot;
use usage::complete::{self, Shell};
#[cfg(feature = "client")]
use usage::spec::{Candidate, CompleteCtx};

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

#[cfg(feature = "client")]
pub(crate) fn complete_target<Partial>(_partial: &Partial, _context: &CompleteCtx<'_>) -> Vec<Candidate<'static>> {
    let mut candidates = vec![
        Candidate::described("next", "the next body"),
        Candidate::described("previous", "the previous body"),
    ];
    candidates.extend(values_of(CompletionKind::Body));
    candidates
}

#[cfg(feature = "client")]
fn values_of(kind: CompletionKind) -> Vec<Candidate<'static>> {
    VALUES.with_borrow(|known| {
        known
            .get(&kind)
            .into_iter()
            .flatten()
            .map(|value| match &value.description {
                Some(description) => Candidate::described(value.value.clone(), description.clone()),
                None => Candidate::new(value.value.clone()),
            })
            .collect()
    })
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
        assert_eq!(completed("camera target ma", &["mars", "mercury"]), ["mars"]);
        let all = completed("camera look-at ", &["earth"]);
        assert!(
            ["next", "previous", "earth"]
                .iter()
                .all(|value| all.contains(&value.to_string()))
        );
    }

    #[test]
    fn the_start_is_where_the_completed_word_begins() {
        assert_eq!(complete("time ra", 7, values(&[])).start, 5);
    }
}
