#[cfg(feature = "client")]
mod body_category_completer;
#[cfg(feature = "client")]
mod body_completer;
mod command_completion;
mod completion_candidate;
mod completion_kind;
mod completions;
mod config_completer;
#[cfg(feature = "client")]
mod coordinate_completer;
#[cfg(feature = "client")]
mod time_of_day_completer;

#[cfg(feature = "client")]
pub(crate) use body_category_completer::complete_body_category;
#[cfg(feature = "client")]
pub(crate) use body_completer::complete_body;
pub use command_completion::CommandCompletion;
pub use completion_candidate::CompletionCandidate;
pub use completion_kind::CompletionKind;
pub use completions::Completions;
pub(crate) use config_completer::{complete_config_key, complete_config_value};
#[cfg(feature = "client")]
pub(crate) use coordinate_completer::{complete_latitude, complete_longitude};
#[cfg(feature = "client")]
pub(crate) use time_of_day_completer::complete_time_of_day;

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    // usage-rs completers are plain functions; the executor's values reach them through here for one call.
    static VALUES: RefCell<HashMap<CompletionKind, Vec<CompletionCandidate>>> = RefCell::new(HashMap::new());
}
