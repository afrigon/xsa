use xsa_proto::network::ClientLink;

pub enum Peer {
    Client(ClientLink),
    Operator(ClientLink),
}
