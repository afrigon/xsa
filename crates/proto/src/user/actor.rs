use super::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Who a connection acts as.
pub enum Actor {
    /// A user who joined the session.
    User(UserId),
    /// The dedicated server's own process, running the commands of its shell, IPC and startup scripts. The server
    /// admits it when it creates its connection; it never joins.
    Operator,
}
