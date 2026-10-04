use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::thread;

use anyhow::Context;
use quinn::crypto::rustls::QuicClientConfig;
use quinn::{Endpoint, RecvStream, SendStream};

use super::fingerprint_verifier::FingerprintVerifier;
use super::stream_pump::StreamPump;
use super::{APPLICATION_PROTOCOL, Fingerprint, SERVER_NAME};
use crate::connection::Connection;

const IPV4_ANY_ADDRESS: &str = "0.0.0.0:0";
const IPV6_ANY_ADDRESS: &str = "[::]:0";

pub(crate) struct RemoteConnection {
    endpoint: Endpoint,
    connection: quinn::Connection,
    send: SendStream,
    receive: RecvStream,
}

impl RemoteConnection {
    pub fn start(address: &str, expected: Fingerprint) -> anyhow::Result<Connection> {
        let socket_address = address
            .to_socket_addrs()
            .with_context(|| format!("resolving {address}"))?
            .next()
            .with_context(|| format!("{address} did not resolve to any address"))?;
        let local = Connection::local();
        let link = local.link;
        let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel::<anyhow::Result<()>>(1);

        thread::Builder::new()
            .name("network".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                    Ok(runtime) => runtime,
                    Err(err) => {
                        let _ = ready_sender.send(Err(err.into()));
                        return;
                    }
                };

                runtime.block_on(async move {
                    match RemoteConnection::open(socket_address, expected).await {
                        Ok(remote) => {
                            let _ = ready_sender.send(Ok(()));
                            let _endpoint = remote.endpoint;
                            let _connection = remote.connection;
                            let _ = StreamPump::new(remote.send, remote.receive)
                                .run(link.messages, link.events)
                                .await;
                        }
                        Err(err) => {
                            let _ = ready_sender.send(Err(err));
                        }
                    }
                });
            })
            .context("starting the network thread")?;

        ready_receiver.recv().context("the network thread stopped")??;

        Ok(local.connection)
    }

    async fn open(address: SocketAddr, expected: Fingerprint) -> anyhow::Result<RemoteConnection> {
        let bind_address: SocketAddr = if address.is_ipv6() {
            IPV6_ANY_ADDRESS.parse()?
        } else {
            IPV4_ANY_ADDRESS.parse()?
        };
        let mut endpoint = Endpoint::client(bind_address)?;
        endpoint.set_default_client_config(RemoteConnection::client_config(expected)?);
        let connection = endpoint
            .connect(address, SERVER_NAME)?
            .await
            .with_context(|| format!("connecting to {address}"))?;
        let (send, receive) = connection.open_bi().await?;

        Ok(RemoteConnection {
            endpoint,
            connection,
            send,
            receive,
        })
    }

    fn client_config(expected: Fingerprint) -> anyhow::Result<quinn::ClientConfig> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let verifier = Arc::new(FingerprintVerifier {
            expected,
            provider: provider.clone(),
        });
        let mut crypto = rustls::ClientConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();
        crypto.alpn_protocols = vec![APPLICATION_PROTOCOL.to_vec()];

        Ok(quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?)))
    }
}
