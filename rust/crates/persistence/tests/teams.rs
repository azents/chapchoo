use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey};
use chapchoo_domain::{EmailAddress, MembershipRole, TeamKind, TeamRole};
use chapchoo_persistence::{
    CreateTeam, InvitationStatus, InviteMember, Store, StoreError, TeamPatch, TeamSignupGrant,
    prepare_postgres, prepare_sqlite,
};
use diesel::{Connection, RunQueryDsl, connection::SimpleConnection, sql_types::BigInt};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::{error::Error, sync::Arc};

const NOW: i64 = 1_000;
type TestError = Box<dyn Error + Send + Sync>;

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|value| format!("{value:02x}")).collect()
}

fn seed_sql(
    cipher: &CredentialCipher,
    tokens: &[BearerSecret],
    postgres: bool,
) -> Result<String, TestError> {
    let mut sql = String::from("BEGIN;");
    for (index, token) in tokens.iter().enumerate() {
        let user = format!("user-{index}");
        let personal = format!("personal-{index}");
        let session = format!("session-{index}");
        let envelope = cipher.protect_latest(&session, token)?;
        let binary = |bytes: &[u8]| {
            if postgres {
                format!("decode('{}','hex')", hex(bytes))
            } else {
                format!("X'{}'", hex(bytes))
            }
        };
        sql.push_str(&format!(
            "INSERT INTO users(id,email_key,delivery_email,password_hash,is_admin,created_at,language,time_zone,region) VALUES('{user}','user{index}@example.invalid','User{index}@Example.Invalid','synthetic',{},0,'en','UTC','en-US');
             INSERT INTO teams VALUES('{personal}','PERSONAL','Personal','{personal}','{user}',0);
             INSERT INTO memberships VALUES('{personal}','{user}','MEMBER');
             INSERT INTO personal_contexts VALUES('{user}','{personal}');
             INSERT INTO auth_sessions(id,user_id,current_hash,recovery_nonce,recovery_ciphertext,created_at,rotated_at,expires_at) VALUES('{session}','{user}',{},{},{},0,0,10000000);",
            match (postgres, index == 3) { (true, true) => "TRUE", (true, false) => "FALSE", (false, true) => "1", (false, false) => "0" }, binary(token.digest().as_bytes()), binary(envelope.nonce()), binary(envelope.ciphertext()),
        ));
    }
    sql.push_str("COMMIT;");
    Ok(sql)
}

async fn exercise(store: Store, tokens: &[BearerSecret]) -> Result<(), TestError> {
    let owner = tokens[0].digest();
    let manager = tokens[1].digest();
    let member = tokens[2].digest();
    let stranger = tokens[3].digest();
    assert!(matches!(
        store
            .create_team(
                &owner,
                &CreateTeam {
                    id: "invalid".to_owned(),
                    name: String::new(),
                    handle: "invalid".to_owned()
                },
                NOW
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    let created = store
        .create_team(
            &owner,
            &CreateTeam {
                id: "team-a".to_owned(),
                name: "Original".to_owned(),
                handle: "team-a".to_owned(),
            },
            NOW,
        )
        .await?;
    assert_eq!(created.caller_role, TeamRole::Owner);
    assert_eq!(created.kind, TeamKind::Collaborative);
    assert!(!store.current_identity(&owner, NOW).await?.is_admin);
    assert!(store.current_identity(&stranger, NOW).await?.is_admin);
    assert_eq!(store.teams(&owner, NOW).await?.len(), 2);
    assert!(matches!(
        store.team(&stranger, "team-a", NOW).await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store.leave_team(&owner, "team-a", NOW).await,
        Err(StoreError::OwnerTransferRequired)
    ));
    assert!(matches!(
        store.delete_team(&owner, "personal-0", NOW).await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .update_team(
                &owner,
                "personal-0",
                &TeamPatch {
                    name: Some("Changed".to_owned()),
                    handle: None
                },
                NOW
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));
    for (id, index) in [("invite-manager", 1), ("invite-member", 2)] {
        let issued = store
            .invite_member(
                &owner,
                &InviteMember {
                    id: id.to_owned(),
                    team_id: "team-a".to_owned(),
                    recipient: EmailAddress::parse(&format!("USER{index}@example.invalid"))?,
                    signup: None,
                },
                NOW,
            )
            .await?;
        assert!(issued.issued_signup_grant_id.is_none());
        let digest = tokens[index].digest();
        assert_eq!(store.received_invitations(&digest, NOW).await?.len(), 1);
        assert!(matches!(
            store.process_invitation(&stranger, id, true, NOW).await,
            Err(StoreError::NotFound)
        ));
        assert_eq!(
            store
                .process_invitation(&digest, id, true, NOW)
                .await?
                .status,
            InvitationStatus::Accepted
        );
        assert!(matches!(
            store.process_invitation(&digest, id, true, NOW).await,
            Err(StoreError::AlreadyProcessed)
        ));
    }
    assert_eq!(store.team_members(&owner, "team-a", NOW).await?.len(), 3);
    assert!(matches!(
        store
            .update_team(
                &member,
                "team-a",
                &TeamPatch {
                    name: Some("Denied".to_owned()),
                    handle: None
                },
                NOW
            )
            .await,
        Err(StoreError::Forbidden)
    ));
    store
        .set_member_role(&owner, "team-a", "user-1", MembershipRole::Manager, NOW)
        .await?;
    assert_eq!(
        store.team(&manager, "team-a", NOW).await?.caller_role,
        TeamRole::Manager
    );
    store
        .set_member_role(&owner, "team-a", "user-1", MembershipRole::Member, NOW)
        .await?;
    assert_eq!(
        store.team(&manager, "team-a", NOW).await?.caller_role,
        TeamRole::Member
    );
    store
        .set_member_role(&owner, "team-a", "user-1", MembershipRole::Manager, NOW)
        .await?;
    assert!(matches!(
        store
            .set_member_role(&manager, "team-a", "user-2", MembershipRole::Manager, NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    assert!(matches!(
        store
            .remove_team_member(&manager, "team-a", "user-0", NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    assert!(matches!(
        store
            .remove_team_member(&owner, "team-a", "user-1", NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    store
        .update_team(
            &manager,
            "team-a",
            &TeamPatch {
                name: Some("Updated".to_owned()),
                handle: Some("updated-handle".to_owned()),
            },
            NOW,
        )
        .await?;
    assert_eq!(store.team(&member, "team-a", NOW).await?.name, "Updated");
    // A failed second patch field rolls back the first field too.
    assert!(matches!(
        store
            .update_team(
                &owner,
                "team-a",
                &TeamPatch {
                    name: Some("Must rollback".to_owned()),
                    handle: Some("personal-0".to_owned())
                },
                NOW
            )
            .await,
        Err(StoreError::HandleConflict)
    ));
    assert_eq!(store.team(&owner, "team-a", NOW).await?.name, "Updated");
    assert!(matches!(
        store
            .invite_member(
                &manager,
                &InviteMember {
                    id: "duplicate-member".to_owned(),
                    team_id: "team-a".to_owned(),
                    recipient: EmailAddress::parse("user2@example.invalid")?,
                    signup: None
                },
                NOW
            )
            .await,
        Err(StoreError::AlreadyMember)
    ));
    // Unknown no-mail recipient gets an invitation but no automatic signup grant.
    let unknown = store
        .invite_member(
            &manager,
            &InviteMember {
                id: "unknown-no-mail".to_owned(),
                team_id: "team-a".to_owned(),
                recipient: EmailAddress::parse("unknown@example.invalid")?,
                signup: None,
            },
            NOW,
        )
        .await?;
    assert!(unknown.issued_signup_grant_id.is_none());
    let proof = BearerSecret::generate()?;
    let unknown_mail = store
        .invite_member(
            &manager,
            &InviteMember {
                id: "unknown-mail".to_owned(),
                team_id: "team-a".to_owned(),
                recipient: EmailAddress::parse("mail@example.invalid")?,
                signup: Some(TeamSignupGrant {
                    id: "issued-grant".to_owned(),
                    proof: proof.digest(),
                    expires_at: NOW + 1000,
                }),
            },
            NOW,
        )
        .await?;
    assert_eq!(
        unknown_mail.issued_signup_grant_id.as_deref(),
        Some("issued-grant")
    );
    // An invitation collision must roll back the newly prepared signup grant.
    assert!(matches!(
        store
            .invite_member(
                &manager,
                &InviteMember {
                    id: "unknown-mail".to_owned(),
                    team_id: "team-a".to_owned(),
                    recipient: EmailAddress::parse("rollback@example.invalid")?,
                    signup: Some(TeamSignupGrant {
                        id: "rolled-back-grant".to_owned(),
                        proof: BearerSecret::generate()?.digest(),
                        expires_at: NOW + 1000
                    })
                },
                NOW
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert_eq!(
        store.team_invitations(&manager, "team-a", NOW).await?.len(),
        4
    );
    assert!(matches!(
        store.team_invitations(&member, "team-a", NOW).await,
        Err(StoreError::Forbidden)
    ));
    let decline = store
        .invite_member(
            &owner,
            &InviteMember {
                id: "decline".to_owned(),
                team_id: "team-a".to_owned(),
                recipient: EmailAddress::parse("user3@example.invalid")?,
                signup: Some(TeamSignupGrant {
                    id: "unused-existing-user-grant".to_owned(),
                    proof: BearerSecret::generate()?.digest(),
                    expires_at: NOW + 1000,
                }),
            },
            NOW,
        )
        .await?;
    assert_eq!(decline.invitation.status, InvitationStatus::Pending);
    assert!(decline.issued_signup_grant_id.is_none());
    assert_eq!(
        store
            .process_invitation(&stranger, "decline", false, NOW)
            .await?
            .status,
        InvitationStatus::Declined
    );
    assert!(matches!(
        store.team(&stranger, "team-a", NOW).await,
        Err(StoreError::NotFound)
    ));
    // Same authenticated recipient races on one proof: exactly one acceptance wins.
    store
        .invite_member(
            &owner,
            &InviteMember {
                id: "race".to_owned(),
                team_id: "team-a".to_owned(),
                recipient: EmailAddress::parse("user3@example.invalid")?,
                signup: None,
            },
            NOW,
        )
        .await?;
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let first_store = store.clone();
    let second_store = store.clone();
    let first_barrier = barrier.clone();
    let second_barrier = barrier;
    let first_digest = tokens[3].digest();
    let second_digest = tokens[3].digest();
    let first = tokio::spawn(async move {
        first_barrier.wait().await;
        first_store
            .process_invitation(&first_digest, "race", true, NOW)
            .await
    });
    let second = tokio::spawn(async move {
        second_barrier.wait().await;
        second_store
            .process_invitation(&second_digest, "race", true, NOW)
            .await
    });
    let results = [first.await?, second.await?];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert!(
        results
            .iter()
            .filter_map(|result| result.as_ref().err())
            .all(|error| matches!(error, StoreError::AlreadyProcessed))
    );
    assert_eq!(store.team_members(&owner, "team-a", NOW).await?.len(), 4);
    store
        .remove_team_member(&manager, "team-a", "user-3", NOW)
        .await?;
    assert!(matches!(
        store.team(&stranger, "team-a", NOW).await,
        Err(StoreError::NotFound)
    ));
    store.leave_team(&member, "team-a", NOW).await?;
    assert!(matches!(
        store.team(&member, "team-a", NOW).await,
        Err(StoreError::NotFound)
    ));
    store
        .invite_member(
            &owner,
            &InviteMember {
                id: "member-rejoin".to_owned(),
                team_id: "team-a".to_owned(),
                recipient: EmailAddress::parse("user2@example.invalid")?,
                signup: None,
            },
            NOW,
        )
        .await?;
    store
        .process_invitation(&member, "member-rejoin", true, NOW)
        .await?;
    let transfer = store.transfer_team(&owner, "team-a", "user-2", NOW).await?;
    assert!(!store.current_identity(&owner, NOW).await?.is_admin);
    assert!(!store.current_identity(&member, NOW).await?.is_admin);
    assert!(store.current_identity(&stranger, NOW).await?.is_admin);
    assert_eq!(transfer.owner_user_id, "user-2");
    assert_eq!(transfer.caller_role, TeamRole::Manager);
    assert_eq!(
        store.team(&member, "team-a", NOW).await?.caller_role,
        TeamRole::Owner
    );
    assert!(matches!(
        store.delete_team(&owner, "team-a", NOW).await,
        Err(StoreError::Forbidden)
    ));
    store.leave_team(&owner, "team-a", NOW).await?;
    assert_eq!(
        store.team(&owner, "personal-0", NOW).await?.kind,
        TeamKind::Personal
    );
    store.leave_team(&manager, "team-a", NOW).await?;
    store.delete_team(&member, "team-a", NOW).await?;
    assert!(matches!(
        store.team(&member, "team-a", NOW).await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store.team(&member, "personal-2", NOW).await?.kind,
        TeamKind::Personal
    );
    store
        .create_team(
            &owner,
            &CreateTeam {
                id: "transfer-race".to_owned(),
                name: "Race".to_owned(),
                handle: "transfer-race".to_owned(),
            },
            NOW,
        )
        .await?;
    store
        .invite_member(
            &owner,
            &InviteMember {
                id: "transfer-race-invite".to_owned(),
                team_id: "transfer-race".to_owned(),
                recipient: EmailAddress::parse("user3@example.invalid")?,
                signup: None,
            },
            NOW,
        )
        .await?;
    store
        .process_invitation(&stranger, "transfer-race-invite", true, NOW)
        .await?;
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let transferring = store.clone();
    let leaving = store.clone();
    let transfer_barrier = barrier.clone();
    let leave_barrier = barrier;
    let owner_digest = tokens[0].digest();
    let target_digest = tokens[3].digest();
    let transfer = tokio::spawn(async move {
        transfer_barrier.wait().await;
        transferring
            .transfer_team(&owner_digest, "transfer-race", "user-3", NOW)
            .await
    });
    let leave = tokio::spawn(async move {
        leave_barrier.wait().await;
        leaving
            .leave_team(&target_digest, "transfer-race", NOW)
            .await
    });
    match (transfer.await?, leave.await?) {
        (Ok(summary), Err(StoreError::OwnerTransferRequired)) => {
            assert_eq!(summary.owner_user_id, "user-3");
            assert_eq!(
                store
                    .team(&stranger, "transfer-race", NOW)
                    .await?
                    .caller_role,
                TeamRole::Owner
            );
            store.delete_team(&stranger, "transfer-race", NOW).await?;
        }
        (Err(StoreError::NotFound), Ok(())) => {
            assert_eq!(
                store
                    .team(&owner, "transfer-race", NOW)
                    .await?
                    .owner_user_id,
                "user-0"
            );
            store.delete_team(&owner, "transfer-race", NOW).await?;
        }
        outcome => return Err(format!("Unexpected transfer/leave ordering: {outcome:?}").into()),
    }
    Ok(())
}

#[test]
fn team_and_invitation_operations_on_sqlite() -> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("teams.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([1; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let tokens = (0..4)
        .map(|_| BearerSecret::generate())
        .collect::<Result<Vec<_>, _>>()?;
    let mut connection =
        diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid fixture path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    connection.batch_execute(&seed_sql(&cipher, &tokens, false)?)?;
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        exercise(Store::connect_sqlite(path.clone(), cipher).await?, &tokens).await
    })?;
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM grants WHERE id='rolled-back-grant'")
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM users")
            .get_result::<Count>(&mut connection)?
            .n,
        4
    );
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM personal_contexts")
            .get_result::<Count>(&mut connection)?
            .n,
        4
    );
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM memberships WHERE team_id='team-a'")
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    assert_eq!(
        diesel::sql_query("SELECT count(*) AS n FROM invitations WHERE team_id='team-a'")
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    Ok(())
}

#[test]
fn team_and_invitation_operations_on_postgres() -> Result<(), TestError> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let unique = BearerSecret::generate()?;
    let suffix = hex(&unique.digest().as_bytes()[..8]);
    let schema = format!("team_test_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
            URL_SAFE.encode([1; 32]),
        )?)?);
        prepare_postgres(&url, &cipher)?;
        let tokens = (0..4)
            .map(|_| BearerSecret::generate())
            .collect::<Result<Vec<_>, _>>()?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        connection.batch_execute(&seed_sql(&cipher, &tokens, true)?)?;
        let runtime = tokio::runtime::Runtime::new()?;
        runtime.block_on(async {
            exercise(Store::connect_postgres(url.clone(), cipher).await?, &tokens).await
        })?;
        assert_eq!(
            diesel::sql_query("SELECT count(*) AS n FROM grants WHERE id='rolled-back-grant'")
                .get_result::<Count>(&mut connection)?
                .n,
            0
        );
        assert_eq!(
            diesel::sql_query("SELECT count(*) AS n FROM users")
                .get_result::<Count>(&mut connection)?
                .n,
            4
        );
        assert_eq!(
            diesel::sql_query("SELECT count(*) AS n FROM personal_contexts")
                .get_result::<Count>(&mut connection)?
                .n,
            4
        );
        assert_eq!(
            diesel::sql_query("SELECT count(*) AS n FROM memberships WHERE team_id='team-a'")
                .get_result::<Count>(&mut connection)?
                .n,
            0
        );
        assert_eq!(
            diesel::sql_query("SELECT count(*) AS n FROM invitations WHERE team_id='team-a'")
                .get_result::<Count>(&mut connection)?
                .n,
            0
        );
        Ok(())
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    result?;
    cleanup?;
    Ok(())
}
