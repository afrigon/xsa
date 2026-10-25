use super::{ClientLink, Connection};

pub struct LocalConnection {
    pub connection: Connection,
    pub link: ClientLink,
}
