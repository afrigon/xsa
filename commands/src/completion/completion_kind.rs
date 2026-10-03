#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionKind {
    Body,
}

impl CompletionKind {
    pub const ALL: [CompletionKind; 1] = [CompletionKind::Body];
}
