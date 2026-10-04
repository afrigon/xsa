use bitcode::{Decode, Encode};

#[derive(Encode, Decode)]
pub(super) enum IpcRequest {
    Execute { words: Vec<String> },
    Complete { line: String, cursor: usize },
}
