use bitcode::{Decode, Encode};

use crate::completion::Completions;

#[derive(Encode, Decode)]
pub(super) enum IpcResponse {
    Output { text: String, succeeded: bool },
    Completions { completions: Completions },
}
