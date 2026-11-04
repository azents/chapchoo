use chapchoo_crypto::SecretDigest;
use chapchoo_domain::{EmailAddress, Handle, MembershipRole, TeamKind, TeamRole};
use diesel::{
    OptionalExtension, QueryableByName,
    sql_types::{BigInt, Binary, Nullable, Text},
};
use diesel_async::RunQueryDsl;

use crate::{
    Store, StoreError,
    entity_id::{EntityId, EntityKind, IdentitySql},
    models::authenticate,
    store::{Dialect, operation},
};

#[derive(Debug)]
pub struct TeamSummary {
    pub id: String,
    pub kind: TeamKind,
    pub name: String,
    pub handle: Option<String>,
    pub owner_user_id: String,
    pub caller_role: TeamRole,
    pub created_at: i64,
}

#[derive(Debug)]
pub struct MemberSummary {
    pub user_id: String,
    pub email: EmailAddress,
    pub role: TeamRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
}

#[derive(Debug)]
pub struct InvitationSummary {
    pub id: String,
    pub team_id: String,
    pub team_name: String,
    pub team_handle: String,
    pub recipient_email: EmailAddress,
    pub invited_by: String,
    pub status: InvitationStatus,
    pub created_at: i64,
    pub processed_at: Option<i64>,
}

#[derive(Debug)]
pub struct CreateTeam {
    pub name: String,
    pub handle: String,
}

#[derive(Debug, Default)]
pub struct TeamPatch {
    pub name: Option<String>,
}

/// Prepared only for the configured-mail branch; no plaintext proof is persisted.
#[derive(Debug)]
pub struct TeamSignupGrant {
    pub proof: SecretDigest,
    pub expires_at: i64,
}

#[derive(Debug)]
pub struct InviteMember {
    pub team_id: String,
    pub recipient: EmailAddress,
    pub signup: Option<TeamSignupGrant>,
}

#[derive(Debug)]
pub struct InvitationResult {
    pub invitation: InvitationSummary,
    /// Present only if this transaction actually issued the prepared grant.
    pub issued_signup_grant_id: Option<String>,
}

#[derive(Debug, diesel::QueryableByName)]
struct TeamRow {
    #[diesel(sql_type = IdentitySql)]
    id: EntityId,
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Nullable<Text>)]
    handle: Option<String>,
    #[diesel(sql_type = IdentitySql)]
    owner_user_id: EntityId,
    #[diesel(sql_type = BigInt)]
    created_at: i64,
}

impl TeamRow {
    fn kind(&self) -> Result<TeamKind, StoreError> {
        match self.kind.as_str() {
            "PERSONAL" => Ok(TeamKind::Personal),
            "COLLABORATIVE" => Ok(TeamKind::Collaborative),
            _ => Err(StoreError::Integrity),
        }
    }

    fn collaborative(&self) -> Result<(), StoreError> {
        if self.kind()? == TeamKind::Collaborative {
            Ok(())
        } else {
            Err(StoreError::InvalidTransition)
        }
    }

    fn summary(
        &self,
        stored_role: MembershipRole,
        user_id: &EntityId,
    ) -> Result<TeamSummary, StoreError> {
        match (self.kind()?, self.handle.as_deref()) {
            (TeamKind::Personal, None) => {}
            (TeamKind::Collaborative, Some(handle)) => {
                Handle::parse(handle).map_err(|_| StoreError::Integrity)?;
            }
            _ => return Err(StoreError::Integrity),
        }
        Ok(TeamSummary {
            id: self.id.to_dto(EntityKind::Team),
            kind: self.kind()?,
            name: self.name.clone(),
            handle: self.handle.clone(),
            owner_user_id: self.owner_user_id.to_dto(EntityKind::User),
            caller_role: TeamRole::effective(stored_role, self.owner_user_id == *user_id),
            created_at: self.created_at,
        })
    }
}

#[derive(Debug, diesel::QueryableByName)]
struct MembershipRow {
    #[diesel(sql_type = Text)]
    role: String,
}

impl MembershipRow {
    fn role(&self) -> Result<MembershipRole, StoreError> {
        match self.role.as_str() {
            "MANAGER" => Ok(MembershipRole::Manager),
            "MEMBER" => Ok(MembershipRole::Member),
            _ => Err(StoreError::Integrity),
        }
    }
}

#[derive(Debug, diesel::QueryableByName)]
struct MemberRow {
    #[diesel(sql_type = IdentitySql)]
    user_id: EntityId,
    #[diesel(sql_type = Text)]
    delivery_email: String,
    #[diesel(sql_type = Text)]
    role: String,
}

#[derive(Debug, diesel::QueryableByName)]
struct TeamMembershipRow {
    #[diesel(embed)]
    team: TeamRow,
    #[diesel(sql_type = Text)]
    caller_stored_role: String,
}

#[derive(Debug, diesel::QueryableByName)]
struct InvitationRow {
    #[diesel(sql_type = IdentitySql)]
    id: EntityId,
    #[diesel(sql_type = IdentitySql)]
    team_id: EntityId,
    #[diesel(sql_type = Text)]
    team_name: String,
    #[diesel(sql_type = Nullable<Text>)]
    team_handle: Option<String>,
    #[diesel(sql_type = Text)]
    recipient_delivery_email: String,
    #[diesel(sql_type = IdentitySql)]
    invited_by: EntityId,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = BigInt)]
    created_at: i64,
    #[diesel(sql_type = Nullable<BigInt>)]
    processed_at: Option<i64>,
}

impl InvitationRow {
    fn summary(self) -> Result<InvitationSummary, StoreError> {
        let status = match self.status.as_str() {
            "PENDING" => InvitationStatus::Pending,
            "ACCEPTED" => InvitationStatus::Accepted,
            "DECLINED" => InvitationStatus::Declined,
            _ => return Err(StoreError::Integrity),
        };
        if (status == InvitationStatus::Pending) != self.processed_at.is_none() {
            return Err(StoreError::Integrity);
        }
        Ok(InvitationSummary {
            id: self.id.to_dto(EntityKind::Invitation),
            team_id: self.team_id.to_dto(EntityKind::Team),
            team_name: self.team_name,
            team_handle: self.team_handle.ok_or(StoreError::Integrity)?,
            recipient_email: EmailAddress::parse(&self.recipient_delivery_email)
                .map_err(|_| StoreError::Integrity)?,
            invited_by: self.invited_by.to_dto(EntityKind::User),
            status,
            created_at: self.created_at,
            processed_at: self.processed_at,
        })
    }
}

#[derive(Debug, diesel::QueryableByName)]
struct IdRow {
    #[diesel(sql_type = IdentitySql)]
    id: EntityId,
}

const TEAM_COLUMNS: &str = "SELECT id,CAST(kind AS TEXT) AS kind,name,handle,owner_user_id,created_at FROM teams WHERE id=? AND deletion_requested_at IS NULL";
const INVITATION_COLUMNS: &str = "SELECT i.id,i.team_id,t.name AS team_name,t.handle AS team_handle,i.recipient_delivery_email,i.invited_by,CAST(i.status AS TEXT) AS status,i.created_at,i.processed_at FROM invitations i JOIN teams t ON t.id=i.team_id AND t.deletion_requested_at IS NULL";

fn conflict(error: diesel::result::Error) -> StoreError {
    match error {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        ) => StoreError::Conflict,
        other => StoreError::Database(other),
    }
}

fn required(value: &str) -> Result<(), StoreError> {
    if value.is_empty() {
        Err(StoreError::InvalidInput)
    } else {
        Ok(())
    }
}

macro_rules! team {
    ($connection:expr, $dialect:expr, $id:expr) => {{
        let sql = match $dialect {
            Dialect::Postgres => format!("{TEAM_COLUMNS} FOR UPDATE"),
            Dialect::Sqlite => TEAM_COLUMNS.to_owned(),
        };
        let mut rows = diesel::sql_query($dialect.query(&sql))
            .bind::<IdentitySql, _>(*$id)
            .load::<TeamRow>($connection)
            .await?;
        if rows.len() != 1 {
            return Err(StoreError::NotFound);
        }
        rows.pop().ok_or(StoreError::NotFound)?
    }};
}

macro_rules! membership {
    ($connection:expr, $dialect:expr, $team:expr, $user:expr) => {{
        let mut rows = diesel::sql_query($dialect.query(
            "SELECT user_id,CAST(role AS TEXT) AS role FROM memberships WHERE team_id=? AND user_id=?",
        ))
        .bind::<IdentitySql, _>(*$team)
        .bind::<IdentitySql, _>(*$user)
        .load::<MembershipRow>($connection)
        .await?;
        rows.pop()
    }};
}

macro_rules! caller {
    ($connection:expr, $dialect:expr, $team:expr, $user:expr) => {{
        let member =
            membership!($connection, $dialect, &$team.id, $user).ok_or(StoreError::NotFound)?;
        TeamRole::effective(member.role()?, $team.owner_user_id == *$user)
    }};
}

fn require_manager(role: TeamRole) -> Result<(), StoreError> {
    match role {
        TeamRole::Owner | TeamRole::Manager => Ok(()),
        TeamRole::Member => Err(StoreError::Forbidden),
    }
}

fn require_owner(role: TeamRole) -> Result<(), StoreError> {
    if role == TeamRole::Owner {
        Ok(())
    } else {
        Err(StoreError::Forbidden)
    }
}

impl Store {
    pub async fn team_handle_available(
        &self,
        actor: &SecretDigest,
        handle: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        self.team_handle_available_scoped(actor, handle, now, false)
            .await
    }

    pub async fn admin_workspace_handle_available(
        &self,
        actor: &SecretDigest,
        handle: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        self.team_handle_available_scoped(actor, handle, now, true)
            .await
    }

    async fn team_handle_available_scoped(
        &self,
        actor: &SecretDigest,
        handle: &str,
        now: i64,
        admin: bool,
    ) -> Result<bool, StoreError> {
        Handle::parse(handle).map_err(|_| StoreError::InvalidInput)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            if admin && !user.is_admin {
                return Err(StoreError::Forbidden);
            }
            let row = diesel::sql_query(dialect.query(
                "SELECT id FROM teams WHERE kind='COLLABORATIVE' AND handle=? AND deletion_requested_at IS NULL",
            )).bind::<Text,_>(handle).get_result::<IdRow>(connection).await.optional()?;
            Ok(row.is_none())
        })
    }

    pub async fn team_by_handle(
        &self,
        actor: &SecretDigest,
        handle: &str,
        now: i64,
    ) -> Result<TeamSummary, StoreError> {
        Handle::parse(handle).map_err(|_| StoreError::InvalidInput)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            let row = diesel::sql_query(dialect.query(
                "SELECT t.id,CAST(t.kind AS TEXT) AS kind,t.name,t.handle,t.owner_user_id,t.created_at,CAST(m.role AS TEXT) AS caller_stored_role
                 FROM teams t JOIN memberships m ON m.team_id=t.id
                 WHERE t.kind='COLLABORATIVE' AND t.handle=? AND t.deletion_requested_at IS NULL AND m.user_id=?",
            )).bind::<Text,_>(handle).bind::<IdentitySql,_>(user_id)
                .get_result::<TeamMembershipRow>(connection).await.optional()?.ok_or(StoreError::NotFound)?;
            let stored = MembershipRow {
                role: row.caller_stored_role,
            }
            .role()?;
            row.team.summary(stored, &user_id)
        })
    }

    pub async fn create_team(
        &self,
        actor: &SecretDigest,
        create: &CreateTeam,
        now: i64,
    ) -> Result<TeamSummary, StoreError> {
        self.create_team_scoped(actor, create, now, false).await
    }

    pub async fn create_admin_workspace(
        &self,
        actor: &SecretDigest,
        create: &CreateTeam,
        now: i64,
    ) -> Result<TeamSummary, StoreError> {
        self.create_team_scoped(actor, create, now, true).await
    }

    async fn create_team_scoped(
        &self,
        actor: &SecretDigest,
        create: &CreateTeam,
        now: i64,
        admin: bool,
    ) -> Result<TeamSummary, StoreError> {
        required(&create.name)?;
        Handle::parse(&create.handle).map_err(|_| StoreError::InvalidInput)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            if admin && !user.is_admin {
                return Err(StoreError::Forbidden);
            }
            let team_id = EntityId::generate(now)?;
            diesel::sql_query(dialect.query("INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES(?,'COLLABORATIVE',?,?,?,?)"))
                .bind::<IdentitySql, _>(team_id)
                .bind::<Text, _>(&create.name)
                .bind::<Text, _>(&create.handle)
                .bind::<IdentitySql, _>(user_id)
                .bind::<BigInt, _>(now)
                .execute(connection).await.map_err(|error| StoreError::unique_field(
                    error, "teams_handle_key", "teams.handle", StoreError::HandleConflict
                ))?;
            diesel::sql_query(
                dialect.query("INSERT INTO memberships(team_id,user_id,role) VALUES(?,?,'MEMBER')"),
            )
            .bind::<IdentitySql, _>(team_id)
            .bind::<IdentitySql, _>(user_id)
            .execute(connection)
            .await
            .map_err(conflict)?;
            let team = team!(connection, dialect, &team_id);
            team.summary(MembershipRole::Member, &user_id)
        })
    }

    pub async fn team(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        now: i64,
    ) -> Result<TeamSummary, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            let team = team!(
                connection,
                dialect,
                &EntityId::from_dto(EntityKind::Team, team_id)?
            );
            let member =
                membership!(connection, dialect, &team.id, &user_id).ok_or(StoreError::NotFound)?;
            team.summary(member.role()?, &user_id)
        })
    }

    pub async fn teams(
        &self,
        actor: &SecretDigest,
        now: i64,
    ) -> Result<Vec<TeamSummary>, StoreError> {
        self.teams_scoped(actor, now, false).await
    }

    pub async fn admin_workspaces(
        &self,
        actor: &SecretDigest,
        now: i64,
    ) -> Result<Vec<TeamSummary>, StoreError> {
        self.teams_scoped(actor, now, true).await
    }

    async fn teams_scoped(
        &self,
        actor: &SecretDigest,
        now: i64,
        admin: bool,
    ) -> Result<Vec<TeamSummary>, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            if admin && !user.is_admin {
                return Err(StoreError::Forbidden);
            }
            let rows = diesel::sql_query(dialect.query("SELECT t.id,CAST(t.kind AS TEXT) AS kind,t.name,t.handle,t.owner_user_id,t.created_at,CAST(m.role AS TEXT) AS caller_stored_role FROM teams t JOIN memberships m ON m.team_id=t.id WHERE m.user_id=? AND t.deletion_requested_at IS NULL ORDER BY t.created_at,t.id"))
                .bind::<IdentitySql, _>(user_id).load::<TeamMembershipRow>(connection).await?;
            rows.into_iter()
                .map(|row| {
                    let stored = MembershipRow {
                        role: row.caller_stored_role,
                    }
                    .role()?;
                    row.team.summary(stored, &user_id)
                })
                .collect()
        })
    }

    pub async fn update_team(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        patch: &TeamPatch,
        now: i64,
    ) -> Result<TeamSummary, StoreError> {
        if let Some(value) = &patch.name {
            required(value)?;
        }
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            let mut team = team!(
                connection,
                dialect,
                &EntityId::from_dto(EntityKind::Team, team_id)?
            );
            let role = caller!(connection, dialect, team, &user_id);
            require_manager(role)?;
            team.collaborative()?;
            if let Some(name) = &patch.name {
                diesel::sql_query(dialect.query("UPDATE teams SET name=? WHERE id=?"))
                    .bind::<Text, _>(name)
                    .bind::<IdentitySql, _>(team.id)
                    .execute(connection)
                    .await?;
                team.name = name.clone();
            }
            let member = membership!(connection, dialect, &team.id, &user_id)
                .ok_or(StoreError::Integrity)?;
            team.summary(member.role()?, &user_id)
        })
    }

    pub async fn team_members(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        now: i64,
    ) -> Result<Vec<MemberSummary>, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            let team = team!(
                connection,
                dialect,
                &EntityId::from_dto(EntityKind::Team, team_id)?
            );
            let _role = caller!(connection, dialect, team, &user_id);
            let members = diesel::sql_query(dialect.query("SELECT m.user_id,u.delivery_email,CAST(m.role AS TEXT) AS role FROM memberships m JOIN users u ON u.id=m.user_id WHERE m.team_id=? ORDER BY u.created_at,u.id"))
                .bind::<IdentitySql, _>(team.id).load::<MemberRow>(connection).await?;
            members
                .into_iter()
                .map(|member| {
                    let stored = MembershipRow { role: member.role }.role()?;
                    Ok(MemberSummary {
                        role: TeamRole::effective(stored, member.user_id == team.owner_user_id),
                        user_id: member.user_id.to_dto(EntityKind::User),
                        email: EmailAddress::parse(&member.delivery_email)
                            .map_err(|_| StoreError::Integrity)?,
                    })
                })
                .collect()
        })
    }

    pub async fn invite_member(
        &self,
        actor: &SecretDigest,
        invite: &InviteMember,
        now: i64,
    ) -> Result<InvitationResult, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            let team = team!(
                connection,
                dialect,
                &EntityId::from_dto(EntityKind::Team, &invite.team_id)?
            );
            require_manager(caller!(connection, dialect, team, &user_id))?;
            team.collaborative()?;
            let recipients =
                diesel::sql_query(dialect.query("SELECT id FROM users WHERE email_key=?"))
                    .bind::<Text, _>(invite.recipient.lookup_key())
                    .load::<IdRow>(connection)
                    .await?;
            if let Some(recipient) = recipients.as_slice().first()
                && membership!(connection, dialect, &team.id, &recipient.id).is_some()
            {
                return Err(StoreError::AlreadyMember);
            }
            let mut issued_signup_grant_id = None;
            if recipients.is_empty()
                && let Some(signup) = &invite.signup
            {
                if signup.expires_at <= now {
                    return Err(StoreError::InvalidProof);
                }
                let signup_id = EntityId::generate(now)?;
                diesel::sql_query(dialect.query("INSERT INTO grants(id,kind,secret_hash,email_key,delivery_email,user_id,issued_by,issued_at,expires_at) VALUES(?,'SIGNUP',?,?,?,NULL,?,?,?)"))
                        .bind::<IdentitySql, _>(signup_id).bind::<Binary, _>(signup.proof.as_bytes().as_slice())
                        .bind::<Text, _>(invite.recipient.lookup_key()).bind::<Text, _>(invite.recipient.delivery())
                        .bind::<IdentitySql, _>(user_id).bind::<BigInt, _>(now).bind::<BigInt, _>(signup.expires_at)
                        .execute(connection).await.map_err(conflict)?;
                issued_signup_grant_id = Some(signup_id.to_dto(EntityKind::Grant));
            }
            let invitation_id = EntityId::generate(now)?;
            diesel::sql_query(dialect.query("INSERT INTO invitations(id,team_id,recipient_email_key,recipient_delivery_email,invited_by,status,created_at,processed_at) VALUES(?,?,?,?,?,'PENDING',?,NULL)"))
                .bind::<IdentitySql, _>(invitation_id).bind::<IdentitySql, _>(team.id)
                .bind::<Text, _>(invite.recipient.lookup_key()).bind::<Text, _>(invite.recipient.delivery())
                .bind::<IdentitySql, _>(user_id).bind::<BigInt, _>(now).execute(connection).await.map_err(conflict)?;
            Ok(InvitationResult {
                invitation: InvitationSummary {
                    id: invitation_id.to_dto(EntityKind::Invitation),
                    team_id: team.id.to_dto(EntityKind::Team),
                    team_name: team.name,
                    team_handle: team.handle.ok_or(StoreError::Integrity)?,
                    recipient_email: invite.recipient.clone(),
                    invited_by: user.id,
                    status: InvitationStatus::Pending,
                    created_at: now,
                    processed_at: None,
                },
                issued_signup_grant_id,
            })
        })
    }

    pub async fn received_invitations(
        &self,
        actor: &SecretDigest,
        now: i64,
    ) -> Result<Vec<InvitationSummary>, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let sql = format!(
                "{INVITATION_COLUMNS} WHERE i.recipient_email_key=? AND i.status='PENDING' ORDER BY i.created_at,i.id"
            );
            let rows = diesel::sql_query(dialect.query(&sql))
                .bind::<Text, _>(user.email.lookup_key())
                .load::<InvitationRow>(connection)
                .await?;
            rows.into_iter().map(InvitationRow::summary).collect()
        })
    }

    pub async fn team_invitations(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        now: i64,
    ) -> Result<Vec<InvitationSummary>, StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            let team = team!(
                connection,
                dialect,
                &EntityId::from_dto(EntityKind::Team, team_id)?
            );
            require_manager(caller!(connection, dialect, team, &user_id))?;
            team.collaborative()?;
            let sql = format!("{INVITATION_COLUMNS} WHERE i.team_id=? ORDER BY i.created_at,i.id");
            let rows = diesel::sql_query(dialect.query(&sql))
                .bind::<IdentitySql, _>(team.id)
                .load::<InvitationRow>(connection)
                .await?;
            rows.into_iter().map(InvitationRow::summary).collect()
        })
    }

    pub async fn process_invitation(
        &self,
        actor: &SecretDigest,
        invitation_id: &str,
        accept: bool,
        now: i64,
    ) -> Result<InvitationSummary, StoreError> {
        let invitation_id = EntityId::from_dto(EntityKind::Invitation, invitation_id)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            // Discover the target without locking an invitation before its Team.
            let targets = diesel::sql_query(dialect.query(
                "SELECT team_id AS id FROM invitations WHERE id=? AND recipient_email_key=?",
            ))
            .bind::<IdentitySql, _>(invitation_id)
            .bind::<Text, _>(user.email.lookup_key())
            .load::<IdRow>(connection)
            .await?;
            let target = targets.as_slice().first().ok_or(StoreError::NotFound)?;
            let team = team!(connection, dialect, &target.id);
            team.collaborative()?;
            let sql = format!("{INVITATION_COLUMNS} WHERE i.id=? AND i.recipient_email_key=?");
            let mut rows = diesel::sql_query(dialect.query(&sql))
                .bind::<IdentitySql, _>(invitation_id)
                .bind::<Text, _>(user.email.lookup_key())
                .load::<InvitationRow>(connection)
                .await?;
            let row = rows.pop().ok_or(StoreError::NotFound)?;
            if row.status != "PENDING" {
                return Err(StoreError::AlreadyProcessed);
            }
            if accept {
                if membership!(connection, dialect, &team.id, &user_id).is_some() {
                    return Err(StoreError::AlreadyMember);
                }
                diesel::sql_query(
                    dialect.query(
                        "INSERT INTO memberships(team_id,user_id,role) VALUES(?,?,'MEMBER')",
                    ),
                )
                .bind::<IdentitySql, _>(team.id)
                .bind::<IdentitySql, _>(user_id)
                .execute(connection)
                .await
                .map_err(conflict)?;
            }
            let update = if accept {
                "UPDATE invitations SET status='ACCEPTED',processed_at=? WHERE id=? AND status='PENDING' AND recipient_email_key=?"
            } else {
                "UPDATE invitations SET status='DECLINED',processed_at=? WHERE id=? AND status='PENDING' AND recipient_email_key=?"
            };
            let changed = diesel::sql_query(dialect.query(update))
                .bind::<BigInt, _>(now)
                .bind::<IdentitySql, _>(invitation_id)
                .bind::<Text, _>(user.email.lookup_key())
                .execute(connection)
                .await?;
            if changed != 1 {
                return Err(StoreError::AlreadyProcessed);
            }
            let mut summary = row.summary()?;
            summary.status = if accept {
                InvitationStatus::Accepted
            } else {
                InvitationStatus::Declined
            };
            summary.processed_at = Some(now);
            Ok(summary)
        })
    }

    pub async fn set_member_role(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        target_user_id: &str,
        role: MembershipRole,
        now: i64,
    ) -> Result<(), StoreError> {
        let target_user_id = EntityId::from_dto(EntityKind::User, target_user_id)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            let team = team!(
                connection,
                dialect,
                &EntityId::from_dto(EntityKind::Team, team_id)?
            );
            require_owner(caller!(connection, dialect, team, &user_id))?;
            team.collaborative()?;
            if target_user_id == team.owner_user_id {
                return Err(StoreError::InvalidTransition);
            }
            membership!(connection, dialect, &team.id, &target_user_id)
                .ok_or(StoreError::NotFound)?;
            let sql = match role {
                MembershipRole::Manager => {
                    "UPDATE memberships SET role='MANAGER' WHERE team_id=? AND user_id=?"
                }
                MembershipRole::Member => {
                    "UPDATE memberships SET role='MEMBER' WHERE team_id=? AND user_id=?"
                }
            };
            diesel::sql_query(dialect.query(sql))
                .bind::<IdentitySql, _>(team.id)
                .bind::<IdentitySql, _>(target_user_id)
                .execute(connection)
                .await?;
            Ok(())
        })
    }

    pub async fn transfer_team(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        target_user_id: &str,
        now: i64,
    ) -> Result<TeamSummary, StoreError> {
        let target_user_id = EntityId::from_dto(EntityKind::User, target_user_id)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            let mut team = team!(
                connection,
                dialect,
                &EntityId::from_dto(EntityKind::Team, team_id)?
            );
            require_owner(caller!(connection, dialect, team, &user_id))?;
            team.collaborative()?;
            if target_user_id == team.owner_user_id {
                return Err(StoreError::InvalidTransition);
            }
            membership!(connection, dialect, &team.id, &target_user_id)
                .ok_or(StoreError::NotFound)?;
            diesel::sql_query(dialect.query("UPDATE teams SET owner_user_id=? WHERE id=?"))
                .bind::<IdentitySql, _>(target_user_id)
                .bind::<IdentitySql, _>(team.id)
                .execute(connection)
                .await?;
            diesel::sql_query(
                dialect
                    .query("UPDATE memberships SET role='MANAGER' WHERE team_id=? AND user_id=?"),
            )
            .bind::<IdentitySql, _>(team.id)
            .bind::<IdentitySql, _>(user_id)
            .execute(connection)
            .await?;
            team.owner_user_id = target_user_id;
            team.summary(MembershipRole::Manager, &user_id)
        })
    }

    pub async fn remove_team_member(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        target_user_id: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        let target_user_id = EntityId::from_dto(EntityKind::User, target_user_id)?;
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            let team = team!(
                connection,
                dialect,
                &EntityId::from_dto(EntityKind::Team, team_id)?
            );
            require_manager(caller!(connection, dialect, team, &user_id))?;
            team.collaborative()?;
            let target = membership!(connection, dialect, &team.id, &target_user_id)
                .ok_or(StoreError::NotFound)?;
            if target_user_id == team.owner_user_id || target.role()? != MembershipRole::Member {
                return Err(StoreError::Forbidden);
            }
            crate::agent_cleanup::delete_team_agents!(
                connection,
                dialect,
                &team.id,
                Some(target_user_id),
                now
            );
            diesel::sql_query(
                dialect.query("DELETE FROM memberships WHERE team_id=? AND user_id=?"),
            )
            .bind::<IdentitySql, _>(team.id)
            .bind::<IdentitySql, _>(target_user_id)
            .execute(connection)
            .await?;
            Ok(())
        })
    }

    pub async fn leave_team(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            let team = team!(
                connection,
                dialect,
                &EntityId::from_dto(EntityKind::Team, team_id)?
            );
            let role = caller!(connection, dialect, team, &user_id);
            team.collaborative()?;
            if role == TeamRole::Owner {
                return Err(StoreError::OwnerTransferRequired);
            }
            crate::agent_cleanup::delete_team_agents!(
                connection,
                dialect,
                &team.id,
                Some(user_id),
                now
            );
            diesel::sql_query(
                dialect.query("DELETE FROM memberships WHERE team_id=? AND user_id=?"),
            )
            .bind::<IdentitySql, _>(team.id)
            .bind::<IdentitySql, _>(user_id)
            .execute(connection)
            .await?;
            Ok(())
        })
    }

    pub async fn delete_team(
        &self,
        actor: &SecretDigest,
        team_id: &str,
        now: i64,
    ) -> Result<(), StoreError> {
        operation!(self, |connection, dialect| {
            let user = authenticate!(connection, dialect, actor, now);
            let user_id = EntityId::from_dto(EntityKind::User, &user.id)?;
            // Only deletion can revisit a pending Team. Retained membership and
            // owner checks still apply before retrying post-commit collection.
            let query = "SELECT id,CAST(kind AS TEXT) AS kind,name,handle,owner_user_id,created_at FROM teams WHERE id=?";
            let query = match dialect {
                Dialect::Postgres => format!("{query} FOR UPDATE"),
                Dialect::Sqlite => query.to_owned(),
            };
            let team = diesel::OptionalExtension::optional(
                diesel::sql_query(dialect.query(&query))
                    .bind::<IdentitySql, _>(EntityId::from_dto(EntityKind::Team, team_id)?)
                    .get_result::<TeamRow>(connection)
                    .await,
            )?
            .ok_or(StoreError::NotFound)?;
            require_owner(caller!(connection, dialect, team, &user_id))?;
            team.collaborative()?;
            crate::agent_cleanup::delete_team_agents!(connection, dialect, &team.id, None, now);
            diesel::sql_query(dialect.query("UPDATE teams SET deletion_requested_at=COALESCE(deletion_requested_at,?) WHERE id=?"))
                .bind::<BigInt, _>(now)
                .bind::<IdentitySql, _>(team.id)
                .execute(connection)
                .await?;
            diesel::sql_query(dialect.query("DELETE FROM invitations WHERE team_id=?"))
                .bind::<IdentitySql, _>(team.id)
                .execute(connection)
                .await?;
            Ok(())
        })?;
        self.collect_deleted_teams().await
    }
}
