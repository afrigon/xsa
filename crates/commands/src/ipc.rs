mod ipc_client;
mod ipc_connection;
mod ipc_endpoint;
mod ipc_request;
mod ipc_response;
mod process_kind;
mod sequence_number;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub use ipc_client::IpcClient;
pub use ipc_endpoint::IpcEndpoint;
pub use process_kind::ProcessKind;

use std::sync::mpsc as std_mpsc;

#[cfg(unix)]
use unix as platform;
#[cfg(windows)]
use windows as platform;

type Bound = std_mpsc::SyncSender<anyhow::Result<IpcEndpoint>>;
