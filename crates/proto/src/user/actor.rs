use super::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    User(UserId),
    Operator,
}
