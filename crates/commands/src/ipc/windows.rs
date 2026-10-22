use std::fs;

use anyhow::Context;
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient, ServerOptions};

use super::ipc_connection::IpcConnection;
use super::{Bound, IpcEndpoint};

const PIPE_DIRECTORY: &str = r"\\.\pipe\";
const PIPE_PREFIX: &str = "xsa-";

fn pipe_name(name: &str) -> String {
    format!("{PIPE_DIRECTORY}{PIPE_PREFIX}{name}")
}

pub async fn serve(name: String, connection: IpcConnection, bound: Bound) {
    let path = pipe_name(&name);
    let mut server = match ServerOptions::new()
        .first_pipe_instance(true)
        .create(&path)
        .with_context(|| format!("an xsa process named {name} is already running"))
    {
        Ok(server) => server,
        Err(err) => {
            let _ = bound.send(Err(err));
            return;
        }
    };
    let _ = bound.send(Ok(IpcEndpoint {}));

    loop {
        if server.connect().await.is_err() {
            return;
        }

        let connected = server;
        server = match ServerOptions::new().create(&path) {
            Ok(server) => server,
            Err(_) => return,
        };
        tokio::spawn(connection.clone().handle(connected));
    }
}

pub async fn connect(name: &str) -> std::io::Result<NamedPipeClient> {
    ClientOptions::new().open(pipe_name(name))
}

pub async fn names() -> anyhow::Result<Vec<String>> {
    let mut names: Vec<String> = fs::read_dir(PIPE_DIRECTORY)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| name.strip_prefix(PIPE_PREFIX).map(str::to_string))
        .collect();
    names.sort();
    Ok(names)
}
