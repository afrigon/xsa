#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NavigationOperation {
    Push,
    Pop,
    SetRoot,
}
