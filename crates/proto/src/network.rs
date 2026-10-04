mod fingerprint;
mod fingerprint_verifier;
mod identity;
mod listener;
mod pending_client;
mod remote_connection;
mod stream_pump;

pub use fingerprint::Fingerprint;
pub use identity::Identity;
pub use listener::Listener;
pub use pending_client::PendingClient;

pub(crate) use remote_connection::RemoteConnection;

const APPLICATION_PROTOCOL: &[u8] = b"xsa/1";
const SERVER_NAME: &str = "xsa";
