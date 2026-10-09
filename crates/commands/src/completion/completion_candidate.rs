use bitcode::{Decode, Encode};
#[cfg(feature = "client")]
use usage::spec::Candidate;

#[cfg(feature = "client")]
use super::{CompletionKind, VALUES};

#[derive(Encode, Decode, Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidate {
    pub value: String,
    pub display: Option<String>,
    pub description: Option<String>,
    // What the value belongs to, such as the config key a config value is for.
    pub scope: Option<String>,
}

impl CompletionCandidate {
    pub fn new(value: impl Into<String>) -> CompletionCandidate {
        CompletionCandidate {
            value: value.into(),
            display: None,
            description: None,
            scope: None,
        }
    }

    #[cfg(feature = "client")]
    pub(crate) fn known(kind: CompletionKind) -> Vec<CompletionCandidate> {
        VALUES.with_borrow(|known| known.get(&kind).cloned().unwrap_or_default())
    }

    #[cfg(feature = "client")]
    pub(crate) fn to_candidate(&self) -> Candidate<'static> {
        let candidate = match &self.description {
            Some(description) => Candidate::described(self.value.clone(), description.clone()),
            None => Candidate::new(self.value.clone()),
        };

        match &self.display {
            Some(display) => candidate.displayed(display.clone()),
            None => candidate,
        }
    }
}
