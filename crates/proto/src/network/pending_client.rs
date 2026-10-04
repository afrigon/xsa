use std::net::SocketAddr;

use super::stream_pump::StreamPump;
use crate::connection::{ClientLink, Connection};

pub struct PendingClient {
    pub(super) incoming: quinn::Incoming,
}

impl PendingClient {
    pub fn remote_address(&self) -> SocketAddr {
        self.incoming.remote_address()
    }

    pub async fn establish(self) -> anyhow::Result<ClientLink> {
        let connection = self.incoming.await?;
        let (send, receive) = connection.accept_bi().await?;
        let local = Connection::local();
        let server_end = local.connection;

        tokio::spawn(async move {
            let _connection = connection;
            let _ = StreamPump::new(send, receive)
                .run(server_end.events, server_end.messages)
                .await;
        });

        Ok(local.link)
    }
}
