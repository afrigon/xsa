use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use quinn::Endpoint;
use quinn::crypto::rustls::QuicServerConfig;
use rustls::pki_types::PrivateKeyDer;

use super::{APPLICATION_PROTOCOL, Identity, PendingClient};

pub struct Listener {
    endpoint: Endpoint,
}

impl Listener {
    pub fn bind(identity: &Identity, address: SocketAddr) -> anyhow::Result<Listener> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let mut crypto = rustls::ServerConfig::builder_with_provider(provider)
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_no_client_auth()
            .with_single_cert(
                vec![identity.certificate.clone()],
                PrivateKeyDer::Pkcs8(identity.private_key.clone_key()),
            )?;
        crypto.alpn_protocols = vec![APPLICATION_PROTOCOL.to_vec()];
        let config = quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(crypto)?));
        let endpoint = Endpoint::server(config, address).with_context(|| format!("listening on {address}"))?;

        Ok(Listener { endpoint })
    }

    pub async fn accept(&self) -> Option<PendingClient> {
        self.endpoint.accept().await.map(|incoming| PendingClient { incoming })
    }
}
