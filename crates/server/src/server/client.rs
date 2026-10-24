use xsa_proto::network::ClientLink;
use xsa_proto::user::Actor;

pub(super) struct Client {
    pub link: ClientLink,
    pub actor: Option<Actor>,
    pub connected: bool,
}

impl Client {
    pub fn new(link: ClientLink, actor: Option<Actor>) -> Client {
        Client {
            link,
            actor,
            connected: true,
        }
    }
}
