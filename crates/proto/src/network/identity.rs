use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

use super::{Fingerprint, SERVER_NAME};

const CERTIFICATE_FILE: &str = "certificate.der";
const PRIVATE_KEY_FILE: &str = "private-key.der";
const OWNER_READ_WRITE: u32 = 0o600;

pub struct Identity {
    pub(super) certificate: CertificateDer<'static>,
    pub(super) private_key: PrivatePkcs8KeyDer<'static>,
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
        Identity::write_private_file(&private_key_path, &private_key)?;

        Ok(Self {
            certificate: generated.cert.der().clone(),
            private_key: PrivatePkcs8KeyDer::from(private_key),
        })
    }

    pub fn fingerprint(&self) -> Fingerprint {
        Fingerprint::of_certificate(&self.certificate)
    }

    fn write_private_file(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, OWNER_READ_WRITE);
        let mut file = options
            .open(path)
            .with_context(|| format!("creating {}", path.display()))?;
        file.write_all(contents)?;

        Ok(())
    }
}
