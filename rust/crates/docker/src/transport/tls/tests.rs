use crate::{
    connection::{Endpoint, Selection, Tls},
    transport::Transport,
};
use rustls::{ServerConfig, pki_types::PrivateKeyDer};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeroize::Zeroizing;

type TestError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::test]
async fn selected_verification_policy_is_preserved_without_plaintext_or_insecure_fallback()
-> Result<(), TestError> {
    for (name, skip, expected) in [
        ("127.0.0.1", false, true),
        ("different.invalid", false, false),
        ("different.invalid", true, true),
    ] {
        let identity = rcgen::generate_simple_self_signed(vec![name.into()])?;
        let ca = identity.cert.pem();
        let key = PrivateKeyDer::Pkcs8(identity.signing_key.serialize_der().into());
        let config =
            ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()?
                .with_no_client_auth()
                .with_single_cert(vec![identity.cert.der().clone()], key)?;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await?;
            let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
            let Ok(mut stream) = acceptor.accept(stream).await else {
                return Ok::<_, TestError>(false);
            };
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                stream.read_exact(&mut byte).await?;
                request.extend_from_slice(&byte);
                if request.len() > 16 * 1024 {
                    return Err("Unbounded TLS fixture request".into());
                }
            }
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
                .await?;
            stream.shutdown().await?;
            Ok(true)
        });
        let selected = Selection {
            api_version: None,
            endpoint: Endpoint::Tcp {
                host: "127.0.0.1".into(),
                port,
            },
            tls: Some(Tls {
                skip_verify: skip,
                ca: Some(Zeroizing::new(ca.into_bytes())),
                certificate: None,
                key: None,
            }),
        };
        let transport = Transport::new(selected).await?;
        assert_eq!(transport.sdk()?.ping().await.is_ok(), expected);
        assert_eq!(server.await??, expected);
        transport.drain().await?;
    }
    Ok(())
}

#[tokio::test]
async fn selected_client_identity_is_required_and_retained_after_source_files_change()
-> Result<(), TestError> {
    use rcgen::{
        BasicConstraints, CertificateParams, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair,
    };
    let mut authority = CertificateParams::new(Vec::<String>::new())?;
    authority.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let authority_key = KeyPair::generate()?;
    let ca = authority.self_signed(&authority_key)?;
    let issuer = Issuer::new(authority, authority_key);
    let mut server_params = CertificateParams::new(vec!["127.0.0.1".into()])?;
    server_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    let server_key = KeyPair::generate()?;
    let server_cert = server_params.signed_by(&server_key, &issuer)?;
    let mut client_params = CertificateParams::new(Vec::<String>::new())?;
    client_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
    let client_key = KeyPair::generate()?;
    let client_cert = client_params.signed_by(&client_key, &issuer)?;
    for provide_identity in [false, true] {
        let mut roots = rustls::RootCertStore::empty();
        roots.add(ca.der().clone())?;
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(
            Arc::new(roots),
            provider.clone(),
        )
        .build()?;
        let config = ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()?
            .with_client_cert_verifier(verifier)
            .with_single_cert(
                vec![server_cert.der().clone()],
                PrivateKeyDer::Pkcs8(server_key.serialize_der().into()),
            )?;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let expected_client = client_cert.der().clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await?;
            let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
            let Ok(mut stream) = acceptor.accept(stream).await else {
                return Ok::<_, TestError>(false);
            };
            assert_eq!(
                stream.get_ref().1.peer_certificates(),
                Some(&[expected_client][..])
            );
            let mut request = Vec::new();
            let mut byte = [0];
            while !request.ends_with(b"\r\n\r\n") {
                stream.read_exact(&mut byte).await?;
                request.extend_from_slice(&byte);
                if request.len() > 16 * 1024 {
                    return Err("Unbounded mutual TLS fixture request".into());
                }
            }
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
                .await?;
            stream.shutdown().await?;
            Ok(true)
        });
        let directory = tempfile::tempdir()?;
        let ca_path = directory.path().join("ca.pem");
        let certificate = directory.path().join("cert.pem");
        let key = directory.path().join("key.pem");
        std::fs::write(&ca_path, ca.pem())?;
        std::fs::write(&certificate, client_cert.pem())?;
        std::fs::write(&key, client_key.serialize_pem())?;
        let explicit = crate::connection::Explicit::from_files(
            format!("tcp://127.0.0.1:{port}"),
            true,
            false,
            Some(&ca_path),
            provide_identity.then_some(certificate.as_path()),
            provide_identity.then_some(key.as_path()),
            None,
        )?;
        let selected = Selection::from_process(Some(explicit))?;
        // Source changes do not change this process's authentication material.
        std::fs::write(&certificate, "replaced client certificate")?;
        std::fs::remove_file(&key)?;
        let transport = Transport::new(selected).await?;
        assert_eq!(transport.sdk()?.ping().await.is_ok(), provide_identity);
        assert_eq!(server.await??, provide_identity);
        transport.drain().await?;
    }
    Ok(())
}
