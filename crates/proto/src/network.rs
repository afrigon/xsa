mod client_link;
mod connection;
mod disconnect_reason;
mod fingerprint;
mod fingerprint_verifier;
mod frame_stream;
mod identity;
mod listener;
mod local_connection;
mod pending_client;
mod received;
mod remote_connection;
mod stream_pump;

pub use client_link::ClientLink;
pub use connection::Connection;
pub use disconnect_reason::DisconnectReason;
pub use fingerprint::Fingerprint;
pub use frame_stream::FrameStream;
pub use identity::Identity;
pub use listener::Listener;
pub use local_connection::LocalConnection;
pub use pending_client::PendingClient;
pub use received::Received;

pub(crate) use remote_connection::RemoteConnection;

const APPLICATION_PROTOCOL: &[u8] = b"xsa/1";
const SERVER_NAME: &str = "xsa";
