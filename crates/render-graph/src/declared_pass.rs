#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DeclaredPass {
    pub name: &'static str,
    pub step_count: u32,
    pub keep: bool,
}
