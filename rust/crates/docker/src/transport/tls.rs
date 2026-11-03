//! User-selected trust/client authentication. Skip verification is constructed
//! only from an explicit selected flag, never after verification failure.
use crate::{Error, connection::Tls};
use rustls::{
    ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature},
    pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime, pem::PemObject},
};
use std::sync::Arc;

#[cfg(test)]
mod tests;

fn certificates(bytes: &[u8]) -> Result<Vec<CertificateDer<'static>>, Error> {
    let certificates = CertificateDer::pem_slice_iter(bytes)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| Error::Configuration)?;
    if certificates.is_empty() {
        return Err(Error::Configuration);
    }
    Ok(certificates)
}

#[derive(Debug)]
struct SelectedSkipVerification(Arc<CryptoProvider>);
impl ServerCertVerifier for SelectedSkipVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        signature: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            signature,
            &self.0.signature_verification_algorithms,
        )
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        signature: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            signature,
            &self.0.signature_verification_algorithms,
        )
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

pub(super) fn configuration(material: &Tls) -> Result<Arc<ClientConfig>, Error> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut roots = RootCertStore::empty();
    if let Some(ca) = &material.ca {
        for certificate in certificates(ca)? {
            roots.add(certificate).map_err(|_| Error::Configuration)?;
        }
    } else if !material.skip_verify {
        for certificate in rustls_native_certs::load_native_certs().certs {
            roots.add(certificate).map_err(|_| Error::Configuration)?;
        }
        if roots.is_empty() {
            return Err(Error::Configuration);
        }
    }
    let builder = ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .map_err(|_| Error::Configuration)?;
    let builder = if material.skip_verify {
        builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(SelectedSkipVerification(provider)))
    } else {
        builder.with_root_certificates(roots)
    };
    let config = match (&material.certificate, &material.key) {
        (Some(certificate), Some(key)) => builder
            .with_client_auth_cert(
                certificates(certificate)?,
                PrivateKeyDer::from_pem_slice(key).map_err(|_| Error::Configuration)?,
            )
            .map_err(|_| Error::Configuration)?,
        (None, None) => builder.with_no_client_auth(),
        _ => return Err(Error::Configuration),
    };
    Ok(Arc::new(config))
}
