#[derive(Clone, Copy)]
pub enum ProcessKind {
    Client,
    Server,
}

impl ProcessKind {
    pub(super) fn default_name(self) -> String {
        let kind = match self {
            ProcessKind::Client => "client",
            ProcessKind::Server => "server",
        };

        format!("{kind}-{}", std::process::id())
    }
}
