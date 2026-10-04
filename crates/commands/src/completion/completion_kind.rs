#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompletionKind {
    Body,
    ConfigKey,
    ConfigValue,
}

impl CompletionKind {
    pub const ALL: [CompletionKind; 3] = [
        CompletionKind::Body,
        CompletionKind::ConfigKey,
        CompletionKind::ConfigValue,
    ];
}
