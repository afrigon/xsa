#[derive(Clone, Copy)]
pub enum InstanceKind {
    Client,
    Server,
}

impl InstanceKind {
    pub(super) fn default_name(self) -> String {
        let kind = match self {
            InstanceKind::Client => "client",
            InstanceKind::Server => "server",
        };

        format!("{kind}-{}", std::process::id())
    }
}
