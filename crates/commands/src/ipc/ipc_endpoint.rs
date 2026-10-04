#[cfg(unix)]
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::thread;

use anyhow::{Context, bail};
use tokio::runtime;
use tokio::sync::mpsc::UnboundedSender;

use super::{InstanceKind, platform};
use crate::router::CommandInvocation;

pub struct IpcEndpoint {
    #[cfg(unix)]
    pub(super) path: PathBuf,
}

impl IpcEndpoint {
    pub fn spawn(
        name: Option<String>,
        kind: InstanceKind,
        invocations: UnboundedSender<CommandInvocation>,
    ) -> anyhow::Result<IpcEndpoint> {
        let name = name.unwrap_or_else(|| kind.default_name());
        IpcEndpoint::validate_name(&name)?;
        let (bound_sender, bound) = std_mpsc::sync_channel(1);

        thread::Builder::new().name("ipc".into()).spawn(move || {
            let runtime = match runtime::Builder::new_current_thread().enable_all().build() {
                Ok(runtime) => runtime,
                Err(err) => {
                    let _ = bound_sender.send(Err(err.into()));
                    return;
                }
            };

            runtime.block_on(platform::serve(name, invocations, bound_sender));
        })?;

        bound.recv().context("the IPC thread stopped before listening")?
    }

    fn validate_name(name: &str) -> anyhow::Result<()> {
        let valid = !name.is_empty()
            && name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_');

        if !valid {
            bail!("{name:?} is not an instance name: use letters, digits, - and _");
        }

        Ok(())
    }
}

#[cfg(unix)]
impl Drop for IpcEndpoint {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_names_are_restricted() {
        assert!(IpcEndpoint::validate_name("client-1234").is_ok());
        assert!(IpcEndpoint::validate_name("agent_1").is_ok());

        for name in ["", "a/b", "../x", "a b", "a.sock"] {
            assert!(IpcEndpoint::validate_name(name).is_err(), "{name:?}");
        }
    }
}
