//! Optional SMTP delivery with mandatory remote TLS and single-attempt submission.
use chapchoo_service::{MailFailure, MailFuture, MailMessage, MailPurpose, Mailer};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox,
    transport::smtp::authentication::Credentials,
};
use std::{fmt, net::IpAddr, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmtpSecurity {
    StartTls,
    ImplicitTls,
    LoopbackPlaintext,
}

/// Configuration errors never echo a host, credential or message body.
#[derive(Debug)]
pub struct SmtpConfigError;

impl fmt::Display for SmtpConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Invalid SMTP configuration")
    }
}
impl std::error::Error for SmtpConfigError {}

pub struct SmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    sender: Mailbox,
}

impl fmt::Debug for SmtpMailer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SmtpMailer([REDACTED])")
    }
}

impl SmtpMailer {
    pub fn new(
        sender: String,
        host: String,
        port: u16,
        security: SmtpSecurity,
        credentials: Option<(String, String)>,
    ) -> Result<Self, SmtpConfigError> {
        if host.is_empty() || host.contains(['/', '@', '\r', '\n']) || port == 0 {
            return Err(SmtpConfigError);
        }
        let sender = sender.parse::<Mailbox>().map_err(|_| SmtpConfigError)?;
        let builder = match security {
            SmtpSecurity::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
                .map_err(|_| SmtpConfigError)?,
            SmtpSecurity::ImplicitTls => {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&host).map_err(|_| SmtpConfigError)?
            }
            SmtpSecurity::LoopbackPlaintext => {
                if credentials.is_some()
                    || !host
                        .parse::<IpAddr>()
                        .is_ok_and(|address| address.is_loopback())
                {
                    return Err(SmtpConfigError);
                }
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
            }
        };
        let mut builder = builder.port(port).timeout(Some(Duration::from_secs(5)));
        if let Some((username, password)) = credentials {
            if username.is_empty() || password.is_empty() {
                return Err(SmtpConfigError);
            }
            builder = builder.credentials(Credentials::new(username, password));
        }
        Ok(Self {
            transport: builder.build(),
            sender,
        })
    }
}

impl Mailer for SmtpMailer {
    fn send(&self, message: MailMessage) -> MailFuture<'_> {
        Box::pin(async move {
            let subject = match message.purpose {
                MailPurpose::LoginCode => "Your Chapchoo sign-in code",
                MailPurpose::SignupLink => "Your Chapchoo signup link",
                MailPurpose::TeamInvitation => "Your Chapchoo team invitation",
            };
            let email = Message::builder()
                .from(self.sender.clone())
                .to(message
                    .to
                    .delivery()
                    .parse::<Mailbox>()
                    .map_err(MailFailure::new)?)
                .subject(subject)
                .body(message.expose_body().to_owned())
                .map_err(MailFailure::new)?;
            // No pooling/reconnect feature and no retry loop: a lost final reply
            // cannot authorize resubmitting a potentially accepted message.
            tokio::time::timeout(Duration::from_secs(15), self.transport.send(email))
                .await
                .map_err(MailFailure::new)?
                .map_err(MailFailure::new)?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    type TestError = Box<dyn Error + Send + Sync>;

    fn message() -> Result<MailMessage, TestError> {
        Ok(MailMessage::new(
            chapchoo_domain::EmailAddress::parse("recipient@example.invalid")?,
            MailPurpose::LoginCode,
            "transient fixture content".to_owned(),
        ))
    }

    #[tokio::test]
    async fn smtp_envelope_body_failure_and_no_automatic_retry() -> Result<(), TestError> {
        for reply in [Some(true), Some(false), None] {
            let success = reply == Some(true);
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
            let port = listener.local_addr()?.port();
            let receiver = tokio::spawn(async move {
                let (stream, _) = listener.accept().await?;
                let mut stream = BufReader::new(stream);
                stream.get_mut().write_all(b"220 fixture ESMTP\r\n").await?;
                let mut commands = Vec::new();
                let mut line = String::new();
                loop {
                    line.clear();
                    if stream.read_line(&mut line).await? == 0 {
                        break;
                    }
                    commands.push(line.clone());
                    if line.starts_with("EHLO ") {
                        stream
                            .get_mut()
                            .write_all(b"250-fixture\r\n250 8BITMIME\r\n")
                            .await?;
                    } else if line.starts_with("MAIL FROM:") || line.starts_with("RCPT TO:") {
                        stream.get_mut().write_all(b"250 OK\r\n").await?;
                    } else if line == "DATA\r\n" {
                        stream.get_mut().write_all(b"354 Send content\r\n").await?;
                        let mut body = String::new();
                        loop {
                            line.clear();
                            stream.read_line(&mut line).await?;
                            if line == ".\r\n" {
                                break;
                            }
                            body.push_str(&line);
                            if body.len() > 32768 {
                                return Err::<_, TestError>("Fixture body exceeded bound".into());
                            }
                        }
                        assert!(body.contains("Your Chapchoo sign-in code"));
                        assert!(body.contains("transient fixture content"));
                        if let Some(accepted) = reply {
                            let response = if accepted {
                                b"250 accepted\r\n".as_slice()
                            } else {
                                b"451 fixture provider secret\r\n".as_slice()
                            };
                            stream.get_mut().write_all(response).await?;
                        }
                        // A dropped final reply is an uncertain acceptance,
                        // not authorization for a second submission.
                        if !success {
                            break;
                        }
                    } else if line == "QUIT\r\n" {
                        stream.get_mut().write_all(b"221 Goodbye\r\n").await?;
                        break;
                    } else {
                        return Err::<_, TestError>("Unexpected SMTP command".into());
                    }
                }
                assert!(
                    commands
                        .iter()
                        .any(|value| value.contains("sender@example.invalid"))
                );
                assert!(
                    commands
                        .iter()
                        .any(|value| value.contains("recipient@example.invalid"))
                );
                Ok::<_, TestError>(listener)
            });
            let mailer = SmtpMailer::new(
                "sender@example.invalid".to_owned(),
                "127.0.0.1".to_owned(),
                port,
                SmtpSecurity::LoopbackPlaintext,
                None,
            )?;
            let result = mailer.send(message()?).await;
            let listener = receiver.await??;
            assert!(
                listener.into_std()?.accept().is_err(),
                "SMTP must not retry submission"
            );
            if success {
                result?;
            } else {
                let error = result.err().ok_or("Expected SMTP failure")?;
                assert!(error.source().is_some());
                assert!(!error.to_string().contains("provider secret"));
                assert!(!format!("{error:?}").contains("provider secret"));
            }
            assert!(!format!("{mailer:?}").contains("sender@example.invalid"));
        }
        Ok(())
    }

    #[tokio::test]
    async fn required_starttls_never_downgrades_or_sends_credentials() -> Result<(), TestError> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let receiver = tokio::spawn(async move {
            let (stream, _) = listener.accept().await?;
            let mut stream = BufReader::new(stream);
            stream.get_mut().write_all(b"220 fixture ESMTP\r\n").await?;
            let mut line = String::new();
            stream.read_line(&mut line).await?;
            assert!(line.starts_with("EHLO "));
            stream
                .get_mut()
                .write_all(b"250-fixture\r\n250 AUTH PLAIN LOGIN\r\n")
                .await?;
            line.clear();
            stream.read_line(&mut line).await?;
            assert!(!line.starts_with("AUTH "));
            assert!(!line.starts_with("MAIL "));
            Ok::<_, TestError>(())
        });
        let mailer = SmtpMailer::new(
            "sender@example.invalid".to_owned(),
            "127.0.0.1".to_owned(),
            port,
            SmtpSecurity::StartTls,
            Some(("fixture-user".to_owned(), "fixture-password".to_owned())),
        )?;
        assert!(mailer.send(message()?).await.is_err());
        receiver.await??;
        Ok(())
    }

    #[test]
    fn plaintext_is_numeric_loopback_only_and_never_authenticated() {
        for host in ["localhost", "smtp.example.invalid", "192.0.2.1"] {
            assert!(
                SmtpMailer::new(
                    "sender@example.invalid".to_owned(),
                    host.to_owned(),
                    25,
                    SmtpSecurity::LoopbackPlaintext,
                    None
                )
                .is_err()
            );
        }
        assert!(
            SmtpMailer::new(
                "sender@example.invalid".to_owned(),
                "127.0.0.1".to_owned(),
                25,
                SmtpSecurity::LoopbackPlaintext,
                Some(("user".to_owned(), "password".to_owned()))
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn authenticated_implicit_tls_sends_with_a_trusted_fixture_certificate()
    -> Result<(), TestError> {
        use lettre::transport::smtp::client::{Certificate, Tls, TlsParameters};
        use std::sync::Arc;
        use tokio_rustls::{
            TlsAcceptor,
            rustls::{ServerConfig, pki_types::PrivatePkcs8KeyDer},
        };
        let certificate = rcgen::generate_simple_self_signed(vec!["localhost".to_owned()])?;
        let tls = TlsParameters::builder("localhost".to_owned())
            .add_root_certificate(Certificate::from_der(certificate.cert.der().to_vec())?)
            .build()?;
        let server = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![certificate.cert.der().clone()],
                PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der()).into(),
            )?;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let receiver = tokio::spawn(async move {
            let (stream, _) = listener.accept().await?;
            let tls_stream = TlsAcceptor::from(Arc::new(server)).accept(stream).await?;
            let mut stream = BufReader::new(tls_stream);
            stream.get_mut().write_all(b"220 fixture ESMTP\r\n").await?;
            let mut line = String::new();
            let mut authenticated = false;
            loop {
                line.clear();
                if stream.read_line(&mut line).await? == 0 {
                    break;
                }
                if line.starts_with("EHLO ") {
                    stream
                        .get_mut()
                        .write_all(b"250-fixture\r\n250 AUTH PLAIN\r\n")
                        .await?;
                } else if line.starts_with("AUTH PLAIN ") {
                    authenticated = true;
                    stream.get_mut().write_all(b"235 Authenticated\r\n").await?;
                } else if line.starts_with("MAIL FROM:") || line.starts_with("RCPT TO:") {
                    assert!(authenticated);
                    stream.get_mut().write_all(b"250 OK\r\n").await?;
                } else if line == "DATA\r\n" {
                    stream.get_mut().write_all(b"354 Send content\r\n").await?;
                    let mut body = String::new();
                    loop {
                        line.clear();
                        stream.read_line(&mut line).await?;
                        if line == ".\r\n" {
                            break;
                        }
                        body.push_str(&line);
                        if body.len() > 32768 {
                            return Err::<_, TestError>("Fixture body exceeded bound".into());
                        }
                    }
                    assert!(body.contains("transient fixture content"));
                    stream.get_mut().write_all(b"250 Accepted\r\n").await?;
                } else if line == "QUIT\r\n" {
                    stream.get_mut().write_all(b"221 Goodbye\r\n").await?;
                    break;
                } else {
                    return Err::<_, TestError>("Unexpected TLS SMTP command".into());
                }
            }
            assert!(authenticated);
            Ok::<_, TestError>(())
        });
        // Custom trust is test-only. Production constructors always use the
        // standard validated roots and expose no certificate-disable setting.
        let mailer = SmtpMailer {
            transport: AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("localhost")
                .port(port)
                .tls(Tls::Wrapper(tls))
                .credentials(Credentials::new(
                    "fixture-user".to_owned(),
                    "fixture-password".to_owned(),
                ))
                .build(),
            sender: "sender@example.invalid".parse()?,
        };
        mailer.send(message()?).await?;
        receiver.await??;
        Ok(())
    }

    #[tokio::test]
    async fn implicit_tls_rejects_an_untrusted_certificate() -> Result<(), TestError> {
        use std::sync::Arc;
        use tokio_rustls::{
            TlsAcceptor,
            rustls::{ServerConfig, pki_types::PrivatePkcs8KeyDer},
        };
        let certificate = rcgen::generate_simple_self_signed(vec!["localhost".to_owned()])?;
        let server = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![certificate.cert.der().clone()],
                PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der()).into(),
            )?;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let receiver = tokio::spawn(async move {
            let (stream, _) = listener.accept().await?;
            let result = TlsAcceptor::from(Arc::new(server)).accept(stream).await;
            assert!(
                result.is_err(),
                "Client must reject the self-signed certificate"
            );
            Ok::<_, TestError>(())
        });
        let mailer = SmtpMailer::new(
            "sender@example.invalid".to_owned(),
            "localhost".to_owned(),
            port,
            SmtpSecurity::ImplicitTls,
            Some(("fixture-user".to_owned(), "fixture-password".to_owned())),
        )?;
        assert!(mailer.send(message()?).await.is_err());
        receiver.await??;
        Ok(())
    }
}
