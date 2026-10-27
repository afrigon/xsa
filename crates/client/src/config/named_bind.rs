use super::Bind;

// A bind as the config names it: `input.bind.<category>.<name>`.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedBind {
    pub category: &'static str,
    pub name: &'static str,
    pub bind: Bind,
}
