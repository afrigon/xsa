use std::fmt;
use std::str::FromStr;

use anyhow::{Context, ensure};
use rustls::pki_types::CertificateDer;

const FINGERPRINT_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fingerprint {
    bytes: [u8; FINGERPRINT_BYTES],
}

impl Fingerprint {
    pub fn of_certificate(certificate: &CertificateDer<'_>) -> Fingerprint {
        let digest = ring::digest::digest(&ring::digest::SHA256, certificate.as_ref());

        Fingerprint {
            bytes: digest.as_ref().try_into().expect("SHA-256 digests are 32 bytes"),
        }
    }
}

impl FromStr for Fingerprint {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> anyhow::Result<Fingerprint> {
        let text = text.trim();
        ensure!(
            text.len() == FINGERPRINT_BYTES * 2,
            "a fingerprint is {} hexadecimal characters",
            FINGERPRINT_BYTES * 2
        );

        let mut bytes = [0; FINGERPRINT_BYTES];

        for (index, byte) in bytes.iter_mut().enumerate() {
            let digits = &text[index * 2..index * 2 + 2];
            *byte = u8::from_str_radix(digits, 16).with_context(|| format!("{digits:?} is not hexadecimal"))?;
        }

        Ok(Fingerprint { bytes })
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.bytes {
            write!(formatter, "{byte:02x}")?;
        }

        Ok(())
    }
}
