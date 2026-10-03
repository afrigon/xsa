use std::fs;
use std::io::Write;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;
use std::thread;

use anyhow::{Context, ensure};
use bitcode::{DecodeOwned, Encode};
use quinn::crypto::rustls::{QuicClientConfig, QuicServerConfig};
use quinn::{Endpoint, RecvStream, SendStream};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::connection::{ClientLink, Connection};
use crate::frame::{read_frame, write_frame};

const APPLICATION_PROTOCOL: &[u8] = b"xsa/1";
const SERVER_NAME: &str = "xsa";
const CERTIFICATE_FILE: &str = "certificate.der";
const PRIVATE_KEY_FILE: &str = "private-key.der";
const FINGERPRINT_BYTES: usize = 32;

type Fingerprint = [u8; FINGERPRINT_BYTES];

pub struct Identity {
    certificate: CertificateDer<'static>,
    private_key: PrivatePkcs8KeyDer<'static>,
}

impl Identity {
    pub fn load_or_generate(directory: &Path) -> anyhow::Result<Self> {
        let certificate_path = directory.join(CERTIFICATE_FILE);
        let private_key_path = directory.join(PRIVATE_KEY_FILE);
        if certificate_path.exists() && private_key_path.exists() {
            return Ok(Self {
                certificate: CertificateDer::from(fs::read(&certificate_path)?),
                private_key: PrivatePkcs8KeyDer::from(fs::read(&private_key_path)?),
            });
        }

        let generated = rcgen::generate_simple_self_signed(vec![SERVER_NAME.to_string()])
            .context("generating the server identity")?;
        let private_key = generated.signing_key.serialize_der();
        fs::create_dir_all(directory).with_context(|| format!("creating {}", directory.display()))?;
        fs::write(&certificate_path, generated.cert.der())?;
        write_private_file(&private_key_path, &private_key)?;
        Ok(Self {
            certificate: generated.cert.der().clone(),
            private_key: PrivatePkcs8KeyDer::from(private_key),
        })
    }

    pub fn fingerprint(&self) -> String {
        fingerprint_of(&self.certificate)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }
}

pub struct Listener {
    endpoint: Endpoint,
}

pub struct PendingClient {
    incoming: quinn::Incoming,
}

pub fn listen(identity: &Identity, address: SocketAddr) -> anyhow::Result<Listener> {
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

impl Listener {
    pub async fn accept(&self) -> Option<PendingClient> {
        self.endpoint.accept().await.map(|incoming| PendingClient { incoming })
    }
}

impl PendingClient {
    pub fn remote_address(&self) -> SocketAddr {
        self.incoming.remote_address()
    }

    pub async fn establish(self) -> anyhow::Result<ClientLink> {
        let connection = self.incoming.await?;
        let (send, receive) = connection.accept_bi().await?;
        let (client_end, link) = Connection::local();
        tokio::spawn(async move {
            let _connection = connection;
            let _ = pump(client_end.events, client_end.messages, send, receive).await;
        });
        Ok(link)
    }
}

pub fn connect(address: &str, fingerprint: &str) -> anyhow::Result<Connection> {
    let expected = parse_fingerprint(fingerprint)?;
    let socket_address = address
        .to_socket_addrs()
        .with_context(|| format!("resolving {address}"))?
        .next()
        .with_context(|| format!("{address} did not resolve to any address"))?;
    let (connection, link) = Connection::local();
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
                match open_connection(socket_address, expected).await {
                    Ok(open) => {
                        let _ = ready_sender.send(Ok(()));
                        let _endpoint = open.endpoint;
                        let _connection = open.connection;
                        let _ = pump(link.messages, link.events, open.send, open.receive).await;
                    }
                    Err(err) => {
                        let _ = ready_sender.send(Err(err));
                    }
                }
            });
        })
        .context("starting the network thread")?;

    ready_receiver.recv().context("the network thread stopped")??;
    Ok(connection)
}

struct OpenConnection {
    endpoint: Endpoint,
    connection: quinn::Connection,
    send: SendStream,
    receive: RecvStream,
}

async fn open_connection(address: SocketAddr, expected: Fingerprint) -> anyhow::Result<OpenConnection> {
    let bind_address: SocketAddr = if address.is_ipv6() {
        "[::]:0".parse()?
    } else {
        "0.0.0.0:0".parse()?
    };
    let mut endpoint = Endpoint::client(bind_address)?;
    endpoint.set_default_client_config(client_config(expected)?);
    let connection = endpoint
        .connect(address, SERVER_NAME)?
        .await
        .with_context(|| format!("connecting to {address}"))?;
    let (send, receive) = connection.open_bi().await?;
    Ok(OpenConnection {
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

async fn pump<Outgoing: Encode, Incoming: DecodeOwned>(
    mut outgoing: UnboundedReceiver<Outgoing>,
    incoming: UnboundedSender<Incoming>,
    mut send: SendStream,
    mut receive: RecvStream,
) -> anyhow::Result<()> {
    let writer = async {
        while let Some(value) = outgoing.recv().await {
            write_frame(&mut send, &value).await?;
        }
        send.finish()?;
        anyhow::Ok(())
    };
    let reader = async {
        while let Some(value) = read_frame(&mut receive).await? {
            if incoming.send(value).is_err() {
                break;
            }
        }
        anyhow::Ok(())
    };
    tokio::select! {
        result = writer => result,
        result = reader => result,
    }
}

#[derive(Debug)]
struct FingerprintVerifier {
    expected: Fingerprint,
    provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for FingerprintVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        if fingerprint_of(end_entity) == self.expected {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General(
                "the server certificate does not match the expected fingerprint".into(),
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        certificate: &CertificateDer<'_>,
        signature: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            certificate,
            signature,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        certificate: &CertificateDer<'_>,
        signature: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            certificate,
            signature,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider.signature_verification_algorithms.supported_schemes()
    }
}

fn fingerprint_of(certificate: &CertificateDer<'_>) -> Fingerprint {
    let digest = ring::digest::digest(&ring::digest::SHA256, certificate.as_ref());
    digest.as_ref().try_into().expect("SHA-256 digests are 32 bytes")
}

fn parse_fingerprint(text: &str) -> anyhow::Result<Fingerprint> {
    let text = text.trim();
    ensure!(
        text.len() == FINGERPRINT_BYTES * 2,
        "a fingerprint is {} hexadecimal characters",
        FINGERPRINT_BYTES * 2
    );
    let mut fingerprint = [0; FINGERPRINT_BYTES];
    for (index, byte) in fingerprint.iter_mut().enumerate() {
        let digits = &text[index * 2..index * 2 + 2];
        *byte = u8::from_str_radix(digits, 16).with_context(|| format!("{digits:?} is not hexadecimal"))?;
    }
    Ok(fingerprint)
}

fn write_private_file(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options
        .open(path)
        .with_context(|| format!("creating {}", path.display()))?;
    file.write_all(contents)?;
    Ok(())
}
