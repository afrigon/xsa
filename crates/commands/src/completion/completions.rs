use std::collections::HashMap;

use bitcode::{Decode, Encode};
use usage::complete::{self, Shell};

use super::{CompletionCandidate, CompletionKind, VALUES};
use crate::command::CommandLine;

// Completion splits a full command line, whose first word is the program.
const PROGRAM_WORD: &str = "xsa ";

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq, Default)]
pub struct Completions {
    pub start: usize,
    pub candidates: Vec<CompletionCandidate>,
}

impl Completions {
    pub fn compute(
        line: &str,
        cursor: usize,
        values: impl Fn(CompletionKind) -> Vec<CompletionCandidate>,
    ) -> Completions {
        let cursor = cursor.min(line.len());
        let full_line = format!("{PROGRAM_WORD}{line}");
        let split = complete::split(&full_line, PROGRAM_WORD.len() + cursor, Shell::Bash);
        VALUES.with_borrow_mut(|known| {
            known.clear();
            known.extend(CompletionKind::ALL.map(|kind| (kind, values(kind))));
        });

        let candidates = complete::candidates(CommandLine::spec(), &split)
            .into_iter()
            .filter(|candidate| candidate.value.starts_with(&split.prefix))
            .map(|candidate| CompletionCandidate {
                value: candidate.value,
                description: candidate.description.map(|description| description.into_owned()),
                scope: None,
            })
            .collect();
        VALUES.with_borrow_mut(HashMap::clear);

        Completions {
            start: cursor.saturating_sub(split.prefix.len()),
            candidates,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(text: &[&str]) -> impl Fn(CompletionKind) -> Vec<CompletionCandidate> {
        let values: Vec<CompletionCandidate> = text.iter().map(|value| CompletionCandidate::new(*value)).collect();

        move |_| values.clone()
    }

    fn completed(line: &str, known: &[&str]) -> Vec<String> {
        Completions::compute(line, line.len(), values(known))
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
        assert_eq!(Completions::compute("time ra", 7, values(&[])).start, 5);
    }
}
