use chapchoo_crypto::{BearerSecret, EmailCode, PasswordHash, SecretDigest};
use chapchoo_domain::{DisplayPatch, DisplaySettings, EmailAddress, MembershipRole, Password};
use chapchoo_persistence::{
    CreateTeam, EmailLoginOutcome, GrantDraft, GrantKind, GrantSummary, InvitationSummary,
    InviteMember, MemberSummary, NewAccount, NewChallenge, NewSession, ProofPreview, SessionPolicy,
    SessionResult, Store, StoreError, TeamPatch, TeamSignupGrant, TeamSummary, UserSummary,
};
use std::{fmt, sync::Arc};
use url::Url;
use zeroize::Zeroizing;

use crate::{
    AcceptanceTime, DomainReason, MailMessage, MailPurpose, Mailer, PasswordWork, ServiceError,
};

const DAY: i64 = 24 * 60 * 60 * 1000;

#[derive(Debug)]
pub struct AccountsConfig {
    pub general_web_url: String,
    pub setup_proof: Option<SecretDigest>,
    pub session_policy: SessionPolicy,
}

#[derive(Debug, Clone, Copy)]
pub struct LoginMethods {
    pub password: bool,
    pub email_code: bool,
    pub signup_email: bool,
}

#[derive(Debug)]
pub struct CodeChallenge {
    pub binding: BearerSecret,
    pub expires_at: i64,
}

pub struct IssuedGrant {
    pub grant: GrantSummary,
    pub proof: BearerSecret,
    url: Zeroizing<String>,
}

impl IssuedGrant {
    pub fn expose_url(&self) -> &str {
        &self.url
    }
}

impl fmt::Debug for IssuedGrant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IssuedGrant")
            .field("grant", &self.grant)
            .field("proof", &"[REDACTED]")
            .field("url", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug)]
pub struct GrantsPage {
    pub items: Vec<GrantSummary>,
    pub total: i64,
}

#[derive(Clone)]
pub struct AccountsService {
    store: Store,
    passwords: PasswordWork,
    mailer: Option<Arc<dyn Mailer>>,
    general_web_url: Url,
    setup_proof: Option<SecretDigest>,
    sessions: SessionPolicy,
}

impl fmt::Debug for AccountsService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AccountsService")
            .field("mail_configured", &self.mailer.is_some())
            .finish_non_exhaustive()
    }
}

impl AccountsService {
    pub fn new(
        store: Store,
        passwords: PasswordWork,
        mailer: Option<Arc<dyn Mailer>>,
        config: AccountsConfig,
    ) -> Result<Self, ServiceError> {
        let general_web_url = Url::parse(&config.general_web_url)
            .map_err(|_| ServiceError::domain(DomainReason::InvalidInput))?;
        let local_host = general_web_url.host_str().is_some_and(|host| {
            host == "localhost"
                || host
                    .trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|address| address.is_loopback())
        });
        if (general_web_url.scheme() == "http" && !local_host)
            || !matches!(general_web_url.scheme(), "http" | "https")
            || general_web_url.host_str().is_none()
            || !general_web_url.username().is_empty()
            || general_web_url.password().is_some()
            || general_web_url.query().is_some()
            || general_web_url.fragment().is_some()
            || config.session_policy.validity_ms <= 0
            || config.session_policy.rotation_ms <= 0
            || config.session_policy.overlap_ms <= 0
            || config
                .session_policy
                .absolute_lifetime_ms
                .is_some_and(|value| value <= 0)
        {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        Ok(Self {
            store,
            passwords,
            mailer,
            general_web_url,
            setup_proof: config.setup_proof,
            sessions: config.session_policy,
        })
    }

    pub fn methods(&self) -> LoginMethods {
        LoginMethods {
            password: true,
            email_code: self.mailer.is_some(),
            signup_email: self.mailer.is_some(),
        }
    }

    pub async fn password_login(
        &self,
        email: &EmailAddress,
        password: Zeroizing<String>,
        clock: AcceptanceTime,
    ) -> Result<SessionResult, ServiceError> {
        let snapshot = self
            .store
            .password_snapshot(email)
            .await?
            .ok_or_else(|| ServiceError::domain(DomainReason::InvalidCredentials))?;
        let verified = self.passwords.verify(snapshot, password).await?;
        let now = clock.now()?;
        self.store
            .accept_password_login(&verified, &new_session()?, now, self.sessions)
            .await
            .map_err(|error| match error {
                StoreError::Unauthenticated => {
                    ServiceError::domain(DomainReason::InvalidCredentials)
                }
                other => other.into(),
            })
    }

    pub async fn send_code(
        &self,
        email: &EmailAddress,
        now: i64,
    ) -> Result<CodeChallenge, ServiceError> {
        let mailer = self.mailer()?;
        let draft = NewChallenge {
            id: id()?,
            email: email.clone(),
            binding: secret()?,
            code: EmailCode::generate().map_err(ServiceError::infrastructure)?,
            expires_at: expiry(now, 10 * 60 * 1000)?,
        };
        self.store.create_login_challenge(&draft, now).await?;
        mailer
            .send(MailMessage::new(
                email.clone(),
                MailPurpose::LoginCode,
                format!(
                    "Your Chapchoo verification code is {}. It expires in 10 minutes.",
                    draft.code.expose()
                ),
            ))
            .await
            .map_err(ServiceError::delivery)?;
        Ok(CodeChallenge {
            binding: draft.binding,
            expires_at: draft.expires_at,
        })
    }

    pub async fn verify_code(
        &self,
        email: &EmailAddress,
        binding: &BearerSecret,
        code: &EmailCode,
        now: i64,
    ) -> Result<SessionResult, ServiceError> {
        self.mailer()?;
        match self
            .store
            .verify_login_challenge(email, binding, code, &new_session()?, now, self.sessions)
            .await
            .map_err(|error| match error {
                StoreError::InvalidProof => ServiceError::domain(DomainReason::InvalidCode),
                other => other.into(),
            })? {
            EmailLoginOutcome::Authenticated(session) => Ok(*session),
            EmailLoginOutcome::RegistrationRequired => {
                Err(ServiceError::domain(DomainReason::RegistrationRequired))
            }
        }
    }

    pub async fn renew(
        &self,
        token: &BearerSecret,
        now: i64,
    ) -> Result<SessionResult, ServiceError> {
        Ok(self.store.renew_session(token, now, self.sessions).await?)
    }

    pub async fn identity(
        &self,
        actor: &SecretDigest,
        now: i64,
    ) -> Result<UserSummary, ServiceError> {
        Ok(self.store.current_identity(actor, now).await?)
    }

    pub async fn logout(&self, actor: &SecretDigest, now: i64) -> Result<(), ServiceError> {
        Ok(self.store.logout_session(actor, now).await?)
    }

    pub async fn display_defaults(&self) -> Result<DisplaySettings, ServiceError> {
        Ok(self.store.display_defaults().await?)
    }
    pub async fn update_display(
        &self,
        actor: &SecretDigest,
        language: Option<&str>,
        time_zone: Option<&str>,
        region: Option<&str>,
        now: i64,
    ) -> Result<UserSummary, ServiceError> {
        Ok(self
            .store
            .update_display(actor, language, time_zone, region, now)
            .await?)
    }
    pub async fn update_display_defaults(
        &self,
        actor: &SecretDigest,
        language: Option<&str>,
        time_zone: Option<&str>,
        region: Option<&str>,
        now: i64,
    ) -> Result<DisplaySettings, ServiceError> {
        Ok(self
            .store
            .update_display_defaults(actor, language, time_zone, region, now)
            .await?)
    }

    pub async fn request_signup_email(
        &self,
        email: &EmailAddress,
        now: i64,
    ) -> Result<(), ServiceError> {
        let mailer = self.mailer()?;
        let proof = secret()?;
        let draft = grant(&proof, expiry(now, 7 * DAY)?)?;
        let url = self.proof_url("signup", &proof)?;
        self.store
            .issue_email_signup_grant(email, &draft, now)
            .await?;
        mailer
            .send(MailMessage::new(
                email.clone(),
                MailPurpose::SignupLink,
                format!("Complete your Chapchoo signup: {}", url.as_str()),
            ))
            .await
            .map_err(ServiceError::delivery)
    }

    pub async fn preview_signup(
        &self,
        proof: &BearerSecret,
        now: i64,
    ) -> Result<Option<ProofPreview>, ServiceError> {
        Ok(self
            .store
            .preview_grant(&proof.digest(), GrantKind::Signup, now)
            .await?)
    }

    pub async fn redeem_signup(
        &self,
        proof: &BearerSecret,
        email: EmailAddress,
        password: Password,
        clock: AcceptanceTime,
        display: Option<DisplayPatch>,
    ) -> Result<SessionResult, ServiceError> {
        let hash = self.passwords.hash(password).await?;
        let draft = account(email, hash, display)?;
        Ok(self
            .store
            .redeem_signup(&proof.digest(), &draft, clock.now()?, self.sessions)
            .await?)
    }

    pub async fn preview_password_reset(
        &self,
        proof: &BearerSecret,
        now: i64,
    ) -> Result<Option<ProofPreview>, ServiceError> {
        Ok(self
            .store
            .preview_grant(&proof.digest(), GrantKind::PasswordReset, now)
            .await?)
    }

    pub async fn redeem_password_reset(
        &self,
        proof: &BearerSecret,
        password: Password,
        clock: AcceptanceTime,
    ) -> Result<(), ServiceError> {
        let hash = self.passwords.hash(password).await?;
        Ok(self
            .store
            .redeem_password_reset(&proof.digest(), &hash, clock.now()?)
            .await?)
    }

    pub async fn setup_status(&self) -> Result<bool, ServiceError> {
        if self.setup_proof.is_none() {
            return Ok(false);
        }
        Ok(self.store.setup_available().await?)
    }

    pub async fn initialize(
        &self,
        installation_proof: &BearerSecret,
        email: EmailAddress,
        password: Password,
        clock: AcceptanceTime,
        display: Option<DisplayPatch>,
    ) -> Result<SessionResult, ServiceError> {
        let configured = self
            .setup_proof
            .as_ref()
            .ok_or_else(|| ServiceError::domain(DomainReason::InvalidProof))?;
        if !configured.matches(installation_proof) {
            return Err(ServiceError::domain(DomainReason::InvalidProof));
        }
        let hash = self.passwords.hash(password).await?;
        let draft = account(email, hash, display)?;
        Ok(self
            .store
            .initialize_admin(
                configured,
                installation_proof,
                &draft,
                clock.now()?,
                self.sessions,
            )
            .await?)
    }

    pub async fn list_teams(
        &self,
        actor: &SecretDigest,
        now: i64,
    ) -> Result<Vec<TeamSummary>, ServiceError> {
        Ok(self.store.teams(actor, now).await?)
    }
    pub async fn get_team(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        now: i64,
    ) -> Result<TeamSummary, ServiceError> {
        Ok(self.store.team(actor, team_id, now).await?)
    }
    pub async fn create_team(
        &self,
        actor: &SecretDigest,
        name: String,
        handle: String,
        now: i64,
    ) -> Result<TeamSummary, ServiceError> {
        Ok(self
            .store
            .create_team(
                actor,
                &CreateTeam {
                    id: id()?,
                    name,
                    handle,
                },
                now,
            )
            .await?)
    }
    pub async fn update_team(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        patch: &TeamPatch,
        now: i64,
    ) -> Result<TeamSummary, ServiceError> {
        Ok(self.store.update_team(actor, team_id, patch, now).await?)
    }
    pub async fn list_members(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        now: i64,
    ) -> Result<Vec<MemberSummary>, ServiceError> {
        Ok(self.store.team_members(actor, team_id, now).await?)
    }
    pub async fn set_member_role(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        target_id: &str,
        role: MembershipRole,
        now: i64,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .set_member_role(actor, team_id, target_id, role, now)
            .await?)
    }
    pub async fn remove_member(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        target_id: &str,
        now: i64,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .remove_team_member(actor, team_id, target_id, now)
            .await?)
    }
    pub async fn transfer_ownership(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        target_id: &str,
        now: i64,
    ) -> Result<(), ServiceError> {
        self.store
            .transfer_team(actor, team_id, target_id, now)
            .await?;
        Ok(())
    }
    pub async fn leave_team(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        now: i64,
    ) -> Result<(), ServiceError> {
        Ok(self.store.leave_team(actor, team_id, now).await?)
    }
    pub async fn delete_team(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        now: i64,
    ) -> Result<(), ServiceError> {
        Ok(self.store.delete_team(actor, team_id, now).await?)
    }

    pub async fn invite_member(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        email: EmailAddress,
        now: i64,
    ) -> Result<InvitationSummary, ServiceError> {
        let proof = if self.mailer.is_some() {
            Some(secret()?)
        } else {
            None
        };
        let signup = proof
            .as_ref()
            .map(|proof| -> Result<TeamSignupGrant, ServiceError> {
                Ok(TeamSignupGrant {
                    id: id()?,
                    proof: proof.digest(),
                    expires_at: expiry(now, 7 * DAY)?,
                })
            })
            .transpose()?;
        let result = self
            .store
            .invite_member(
                actor,
                &InviteMember {
                    id: id()?,
                    team_id: team_id.to_owned(),
                    recipient: email.clone(),
                    signup,
                },
                now,
            )
            .await?;
        if let Some(mailer) = &self.mailer {
            let link = if result.issued_signup_grant_id.is_some() {
                let proof = proof
                    .as_ref()
                    .ok_or_else(|| ServiceError::domain(DomainReason::Internal))?;
                self.proof_url("signup", proof)?
            } else {
                self.public_route("login")?
            };
            mailer
                .send(MailMessage::new(
                    email,
                    MailPurpose::TeamInvitation,
                    format!(
                        "You have been invited to the Chapchoo team {}. Sign in or register: {}",
                        result.invitation.team_name,
                        link.as_str()
                    ),
                ))
                .await
                .map_err(ServiceError::delivery)?;
        }
        Ok(result.invitation)
    }

    pub async fn received_invitations(
        &self,
        actor: &SecretDigest,
        now: i64,
    ) -> Result<Vec<InvitationSummary>, ServiceError> {
        Ok(self.store.received_invitations(actor, now).await?)
    }
    pub async fn team_invitations(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        now: i64,
    ) -> Result<Vec<InvitationSummary>, ServiceError> {
        Ok(self.store.team_invitations(actor, team_id, now).await?)
    }
    pub async fn process_invitation(
        &self,
        actor: &SecretDigest,
        invitation_id: &str,
        accept: bool,
        now: i64,
    ) -> Result<(), ServiceError> {
        self.store
            .process_invitation(actor, invitation_id, accept, now)
            .await?;
        Ok(())
    }

    pub async fn issue_signup_grant(
        &self,
        actor: &SecretDigest,
        email: &EmailAddress,
        now: i64,
    ) -> Result<IssuedGrant, ServiceError> {
        let proof = secret()?;
        let draft = grant(&proof, expiry(now, 7 * DAY)?)?;
        let grant = self
            .store
            .issue_signup_grant(actor, email, &draft, now)
            .await?;
        let url = self.proof_url("signup", &proof)?;
        Ok(IssuedGrant {
            grant,
            proof,
            url: Zeroizing::new(url.to_string()),
        })
    }

    pub async fn issue_password_reset(
        &self,
        actor: &SecretDigest,
        email: &EmailAddress,
        now: i64,
    ) -> Result<IssuedGrant, ServiceError> {
        let proof = secret()?;
        let draft = grant(&proof, expiry(now, DAY)?)?;
        let grant = self
            .store
            .issue_password_reset(actor, email, &draft, now)
            .await?;
        let url = self.proof_url("reset-password", &proof)?;
        Ok(IssuedGrant {
            grant,
            proof,
            url: Zeroizing::new(url.to_string()),
        })
    }

    pub async fn list_signup_grants(
        &self,
        actor: &SecretDigest,
        offset: i64,
        limit: i64,
        now: i64,
    ) -> Result<GrantsPage, ServiceError> {
        self.list_grants(actor, GrantKind::Signup, offset, limit, now)
            .await
    }
    pub async fn list_password_resets(
        &self,
        actor: &SecretDigest,
        offset: i64,
        limit: i64,
        now: i64,
    ) -> Result<GrantsPage, ServiceError> {
        self.list_grants(actor, GrantKind::PasswordReset, offset, limit, now)
            .await
    }
    pub async fn revoke_signup_grant(
        &self,
        actor: &SecretDigest,
        grant_id: &str,
        now: i64,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .revoke_grant(actor, grant_id, GrantKind::Signup, now)
            .await?)
    }
    pub async fn revoke_password_reset(
        &self,
        actor: &SecretDigest,
        grant_id: &str,
        now: i64,
    ) -> Result<(), ServiceError> {
        Ok(self
            .store
            .revoke_grant(actor, grant_id, GrantKind::PasswordReset, now)
            .await?)
    }

    async fn list_grants(
        &self,
        actor: &SecretDigest,
        kind: GrantKind,
        offset: i64,
        limit: i64,
        now: i64,
    ) -> Result<GrantsPage, ServiceError> {
        if offset < 0 || !(1..=1000).contains(&limit) {
            return Err(ServiceError::domain(DomainReason::InvalidInput));
        }
        let (items, total) = self
            .store
            .list_grants(actor, kind, offset, limit, now)
            .await?;
        Ok(GrantsPage { items, total })
    }

    fn mailer(&self) -> Result<&Arc<dyn Mailer>, ServiceError> {
        self.mailer
            .as_ref()
            .ok_or_else(|| ServiceError::domain(DomainReason::MailUnavailable))
    }

    fn public_route(&self, path: &str) -> Result<Url, ServiceError> {
        let mut url = self.general_web_url.clone();
        let prefix = url.path().trim_end_matches('/').to_owned();
        url.set_path(&format!("{prefix}/{path}"));
        Ok(url)
    }

    fn proof_url(&self, path: &str, proof: &BearerSecret) -> Result<Url, ServiceError> {
        let mut url = self.public_route(path)?;
        url.query_pairs_mut().append_pair("token", proof.expose());
        Ok(url)
    }
}

fn secret() -> Result<BearerSecret, ServiceError> {
    BearerSecret::generate().map_err(ServiceError::infrastructure)
}
fn id() -> Result<String, ServiceError> {
    Ok(secret()?
        .digest()
        .as_bytes()
        .iter()
        .map(|value| format!("{value:02x}"))
        .collect())
}
fn expiry(now: i64, duration: i64) -> Result<i64, ServiceError> {
    now.checked_add(duration)
        .ok_or_else(|| ServiceError::domain(DomainReason::InvalidInput))
}
fn grant(proof: &BearerSecret, expires_at: i64) -> Result<GrantDraft, ServiceError> {
    Ok(GrantDraft {
        id: id()?,
        proof: proof.digest(),
        expires_at,
    })
}
fn new_session() -> Result<NewSession, ServiceError> {
    Ok(NewSession {
        id: id()?,
        token: secret()?,
    })
}
fn account(
    email: EmailAddress,
    password: PasswordHash,
    display: Option<DisplayPatch>,
) -> Result<NewAccount, ServiceError> {
    let personal_team_id = id()?;
    Ok(NewAccount {
        user_id: id()?,
        email,
        password,
        personal_handle: format!("personal-{personal_team_id}"),
        personal_team_id,
        session: new_session()?,
        display,
    })
}
