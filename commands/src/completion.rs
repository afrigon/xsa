mod command_completion;
mod completion_candidate;
mod completion_kind;
mod completions;
#[cfg(feature = "client")]
mod target_completer;

pub use command_completion::CommandCompletion;
pub use completion_candidate::CompletionCandidate;
pub use completion_kind::CompletionKind;
pub use completions::Completions;
#[cfg(feature = "client")]
pub(crate) use target_completer::complete_target;

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    // usage-rs completers are plain functions; the executor's values reach them through here for one call.
    static VALUES: RefCell<HashMap<CompletionKind, Vec<CompletionCandidate>>> = RefCell::new(HashMap::new());
}
