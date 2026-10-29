//! Optional Amazon SES delivery through the official provider SDK.
use aws_sdk_sesv2::{
    Client,
    config::{BehaviorVersion, Region, retry::RetryConfig, timeout::TimeoutConfig},
    types::{Body, Content, Destination, EmailContent, Message},
};
use chapchoo_service::{MailFailure, MailFuture, MailMessage, MailPurpose, Mailer};
use std::{fmt, time::Duration};

pub struct SesMailer {
    client: Client,
    sender: String,
}

impl fmt::Debug for SesMailer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SesMailer([REDACTED])")
    }
}

impl SesMailer {
    /// Uses the SDK's ordinary credential chain. Endpoint override is explicit operator config.
    pub async fn new(sender: String, region: String, endpoint: Option<String>) -> Self {
        let shared = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(region))
            // A provider timeout does not prove that no mail was accepted.
            .retry_config(RetryConfig::standard().with_max_attempts(1))
            .timeout_config(
                TimeoutConfig::builder()
                    .operation_timeout(Duration::from_secs(15))
                    .connect_timeout(Duration::from_secs(5))
                    .build(),
            )
            .load()
            .await;
        let mut config = aws_sdk_sesv2::config::Builder::from(&shared);
        if let Some(endpoint) = endpoint {
            config = config.endpoint_url(endpoint);
        }
        Self {
            client: Client::from_conf(config.build()),
            sender,
        }
    }
}

impl Mailer for SesMailer {
    fn send(&self, message: MailMessage) -> MailFuture<'_> {
        Box::pin(async move {
            let subject = match message.purpose {
                MailPurpose::LoginCode => "Your Chapchoo sign-in code",
                MailPurpose::SignupLink => "Your Chapchoo signup link",
                MailPurpose::TeamInvitation => "Your Chapchoo team invitation",
            };
            let subject = Content::builder()
                .data(subject)
                .charset("UTF-8")
                .build()
                .map_err(MailFailure::new)?;
            let text = Content::builder()
                .data(message.expose_body())
                .charset("UTF-8")
                .build()
                .map_err(MailFailure::new)?;
            let content = Message::builder()
                .subject(subject)
                .body(Body::builder().text(text).build())
                .build();
            self.client
                .send_email()
                .from_email_address(&self.sender)
                .destination(
                    Destination::builder()
                        .to_addresses(message.to.delivery())
                        .build(),
                )
                .content(EmailContent::builder().simple(content).build())
                .send()
                .await
                .map_err(MailFailure::new)?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    type TestError = Box<dyn Error + Send + Sync>;

    #[tokio::test]
    async fn official_sdk_serializes_delivery_and_preserves_safe_provider_failure()
    -> Result<(), TestError> {
        for success in [true, false] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
            let address = listener.local_addr()?;
            let receiver = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await?;
                let mut input = Vec::new();
                let mut byte = [0; 1];
                while !input.ends_with(b"\r\n\r\n") {
                    stream.read_exact(&mut byte).await?;
                    input.push(byte[0]);
                    if input.len() > 32_768 {
                        return Err::<_, TestError>("SDK header exceeded fixture bound".into());
                    }
                }
                let header = std::str::from_utf8(&input)?.to_ascii_lowercase();
                assert!(header.starts_with("post /v2/email/outbound-emails "));
                assert!(header.contains("authorization: aws4-hmac-sha256"));
                let length = header
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length: "))
                    .ok_or("Missing SDK request length")?
                    .trim()
                    .parse::<usize>()?;
                let mut body = vec![0; length];
                stream.read_exact(&mut body).await?;
                let body = std::str::from_utf8(&body)?;
                assert!(body.contains("Your Chapchoo sign-in code"));
                assert!(body.contains("transient fixture content"));
                let (status, response) = if success {
                    ("200 OK", "{\"MessageId\":\"fixture\"}")
                } else {
                    (
                        "503 Service Unavailable",
                        "{\"message\":\"provider fixture secret\"}",
                    )
                };
                stream.write_all(format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{response}",
                    response.len()
                ).as_bytes()).await?;
                Ok::<_, TestError>(listener)
            });
            let config = aws_sdk_sesv2::config::Builder::new()
                .behavior_version(BehaviorVersion::latest())
                .region(Region::new("us-east-1"))
                .credentials_provider(aws_sdk_sesv2::config::Credentials::new(
                    "fixture", "fixture", None, None, "fixture",
                ))
                .endpoint_url(format!("http://{address}"))
                .retry_config(RetryConfig::standard().with_max_attempts(1))
                .build();
            let mailer = SesMailer {
                client: Client::from_conf(config),
                sender: "sender@example.invalid".to_owned(),
            };
            let result = tokio::time::timeout(
                Duration::from_secs(10),
                mailer.send(MailMessage::new(
                    chapchoo_domain::EmailAddress::parse("recipient@example.invalid")?,
                    MailPurpose::LoginCode,
                    "transient fixture content".to_owned(),
                )),
            )
            .await?;
            let listener = receiver.await??;
            assert!(
                listener.into_std()?.accept().is_err(),
                "Email submission must not auto-retry"
            );
            if success {
                result?;
            } else {
                let error = result.err().ok_or("Expected provider failure")?;
                assert!(error.source().is_some());
                assert!(!error.to_string().contains("fixture secret"));
                assert!(!format!("{error:?}").contains("fixture secret"));
            }
        }
        Ok(())
    }
}
