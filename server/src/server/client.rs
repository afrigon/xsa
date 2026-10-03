use xsa_proto::connection::ClientLink;

use super::membership::Membership;

pub(super) struct Client {
    pub link: ClientLink,
    pub membership: Option<Membership>,
    pub connected: bool,
}

impl Client {
    pub fn new(link: ClientLink) -> Client {
        Client {
            link,
            membership: None,
            connected: true,
        }
    }
}
