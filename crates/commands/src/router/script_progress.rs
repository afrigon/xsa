#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScriptProgress {
    Progressed,
    Waiting,
    Finished,
}
