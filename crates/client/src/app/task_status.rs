pub(super) enum TaskStatus {
    Pending,
    Done(anyhow::Result<String>),
}
