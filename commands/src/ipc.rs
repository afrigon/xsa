mod instance_kind;
mod ipc_client;
mod ipc_connection;
mod ipc_endpoint;
mod ipc_request;
mod ipc_response;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub use instance_kind::InstanceKind;
pub use ipc_client::IpcClient;
pub use ipc_endpoint::IpcEndpoint;

use std::sync::mpsc as std_mpsc;

#[cfg(unix)]
use unix as platform;
#[cfg(windows)]
use windows as platform;

type Bound = std_mpsc::SyncSender<anyhow::Result<IpcEndpoint>>;
