use chapchoo_domain::EmailAddress;
use std::{error::Error, fmt, future::Future, pin::Pin};
use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailPurpose {
    LoginCode,
    SignupLink,
    TeamInvitation,
}

/// Secret-bearing content is accessible only by the delivery capability.
pub struct MailMessage {
    pub to: EmailAddress,
    pub purpose: MailPurpose,
    body: Zeroizing<String>,
}

impl MailMessage {
    pub fn new(to: EmailAddress, purpose: MailPurpose, body: String) -> Self {
        Self {
            to,
            purpose,
            body: Zeroizing::new(body),
        }
    }

    pub fn expose_body(&self) -> &str {
        &self.body
    }
}

impl fmt::Debug for MailMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MailMessage")
            .field("purpose", &self.purpose)
            .field("body", &"[REDACTED]")
            .finish()
    }
}

pub struct MailFailure {
    source: Box<dyn Error + Send + Sync>,
}

impl MailFailure {
    pub fn new(source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            source: Box::new(source),
        }
    }
}

impl fmt::Debug for MailFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MailFailure([REDACTED])")
    }
}

impl fmt::Display for MailFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Email provider request failed")
    }
}

impl Error for MailFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

pub type MailFuture<'a> = Pin<Box<dyn Future<Output = Result<(), MailFailure>> + Send + 'a>>;

/// Configuration chooses an optional real mail adapter; production has no capture fallback.
pub trait Mailer: Send + Sync {
    fn send(&self, message: MailMessage) -> MailFuture<'_>;
}
