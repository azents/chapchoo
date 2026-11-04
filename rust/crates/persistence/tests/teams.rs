mod support;
#[path = "support/tasks.rs"]
mod tasks;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, PasswordCost, PasswordHasher, RootKey};
use chapchoo_domain::{EmailAddress, MembershipRole, Password, TeamKind, TeamRole};
use chapchoo_persistence::{
    CreateTeam, GrantDraft, InvitationStatus, InviteMember, NewAccount, NewSession, SessionPolicy,
    Store, StoreError, TeamPatch, TeamSignupGrant, prepare_postgres, prepare_sqlite,
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

async fn seed(store: &Store, tokens: &[BearerSecret]) -> Result<(), TestError> {
    let policy = SessionPolicy::default();
    for index in [3, 0, 1, 2] {
        let proof = BearerSecret::generate()?;
        let email = EmailAddress::parse(&format!("User{index}@Example.Invalid"))?;
        let draft = NewAccount {
            email: email.clone(),
            password: PasswordHasher::new(PasswordCost::new(10)?)
                .hash(&Password::parse("Aa1!fixture".to_owned())?)?,
            session: NewSession {
                token: BearerSecret::parse(tokens[index].expose().to_owned())?,
            },
            display: None,
        };
        if index == 3 {
            store
                .initialize_admin(&proof.digest(), &proof, &draft, NOW, policy)
                .await?;
        } else {
            store
                .issue_email_signup_grant(
                    &email,
                    &GrantDraft {
                        proof: proof.digest(),
                        expires_at: NOW + 10000,
                    },
                    NOW,
                )
                .await?;
            store
                .redeem_signup(&proof.digest(), &draft, NOW, policy)
                .await?;
        }
    }
    Ok(())
}

async fn exercise(
    store: &Store,
    tokens: &[BearerSecret],
    connection: &mut impl SimpleConnection,
    postgres: bool,
) -> Result<(), TestError> {
    let owner = tokens[0].digest();
    let manager = tokens[1].digest();
    let member = tokens[2].digest();
    let stranger = tokens[3].digest();
    let mut users = Vec::new();
    let mut personals = Vec::new();
    for token in tokens {
        let actor = token.digest();
        users.push(store.current_identity(&actor, NOW).await?.id);
        let personal = store
            .teams(&actor, NOW)
            .await?
            .into_iter()
            .find(|team| team.kind == TeamKind::Personal)
            .ok_or("Missing personal Team")?;
        assert!(personal.handle.is_none());
        personals.push(personal.id);
    }
    assert!(matches!(
        store
            .create_team(
                &owner,
                &CreateTeam {
                    name: String::new(),
                    handle: "invalid".to_owned()
                },
                NOW
            )
            .await,
        Err(StoreError::InvalidInput)
    ));
    for handle in ["ab", "Upper", "a_b", "abc\n", "abcdefghijklmnopq"] {
        assert!(matches!(
            store
                .create_team(
                    &owner,
                    &CreateTeam {
                        name: "Invalid".to_owned(),
                        handle: handle.to_owned()
                    },
                    NOW
                )
                .await,
            Err(StoreError::InvalidInput)
        ));
    }
    let created = store
        .create_team(
            &owner,
            &CreateTeam {
                name: "Original".to_owned(),
                handle: "team-a".to_owned(),
            },
            NOW,
        )
        .await?;
    let team = created.id;
    assert_eq!(created.caller_role, TeamRole::Owner);
    assert_eq!(created.kind, TeamKind::Collaborative);
    assert_eq!(created.handle.as_deref(), Some("team-a"));
    assert!(!store.current_identity(&owner, NOW).await?.is_admin);
    assert!(store.current_identity(&stranger, NOW).await?.is_admin);
    assert_eq!(store.teams(&owner, NOW).await?.len(), 2);
    let admin_contexts = store.admin_workspaces(&stranger, NOW).await?;
    assert_eq!(admin_contexts.len(), 1);
    assert_eq!(admin_contexts[0].id, personals[3]);
    let admin_create = CreateTeam {
        name: "Administrator's own Team".to_owned(),
        handle: "admin-created".to_owned(),
    };
    assert!(matches!(
        store.admin_workspaces(&owner, NOW).await,
        Err(StoreError::Forbidden)
    ));
    assert!(matches!(
        store
            .create_admin_workspace(&owner, &admin_create, NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    let admin_team = store
        .create_admin_workspace(&stranger, &admin_create, NOW)
        .await?;
    assert_eq!(admin_team.owner_user_id, users[3]);
    assert_eq!(store.admin_workspaces(&stranger, NOW).await?.len(), 2);
    assert_eq!(store.teams(&owner, NOW).await?.len(), 2);
    assert!(matches!(
        store
            .create_team(
                &owner,
                &CreateTeam {
                    name: "Collision".to_owned(),
                    handle: "team-a".to_owned()
                },
                NOW
            )
            .await,
        Err(StoreError::HandleConflict)
    ));
    assert!(matches!(
        store.team(&stranger, &team, NOW).await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store.leave_team(&owner, &team, NOW).await,
        Err(StoreError::OwnerTransferRequired)
    ));
    assert!(matches!(
        store.delete_team(&owner, &personals[0], NOW).await,
        Err(StoreError::InvalidTransition)
    ));
    assert!(matches!(
        store
            .update_team(
                &owner,
                &personals[0],
                &TeamPatch {
                    name: Some("Changed".to_owned())
                },
                NOW
            )
            .await,
        Err(StoreError::InvalidTransition)
    ));

    for index in [1, 2] {
        let issued = store
            .invite_member(
                &owner,
                &InviteMember {
                    team_id: team.clone(),
                    recipient: EmailAddress::parse(&format!("USER{index}@example.invalid"))?,
                    signup: None,
                },
                NOW,
            )
            .await?;
        assert!(issued.issued_signup_grant_id.is_none());
        let id = issued.invitation.id;
        let digest = tokens[index].digest();
        assert_eq!(store.received_invitations(&digest, NOW).await?.len(), 1);
        assert!(matches!(
            store.process_invitation(&stranger, &id, true, NOW).await,
            Err(StoreError::NotFound)
        ));
        assert_eq!(
            store
                .process_invitation(&digest, &id, true, NOW)
                .await?
                .status,
            InvitationStatus::Accepted
        );
        assert!(matches!(
            store.process_invitation(&digest, &id, true, NOW).await,
            Err(StoreError::AlreadyProcessed)
        ));
    }
    assert_eq!(store.team_members(&owner, &team, NOW).await?.len(), 3);
    assert!(matches!(
        store
            .update_team(
                &member,
                &team,
                &TeamPatch {
                    name: Some("Denied".to_owned())
                },
                NOW
            )
            .await,
        Err(StoreError::Forbidden)
    ));
    for (role, expected) in [
        (MembershipRole::Manager, TeamRole::Manager),
        (MembershipRole::Member, TeamRole::Member),
        (MembershipRole::Manager, TeamRole::Manager),
    ] {
        store
            .set_member_role(&owner, &team, &users[1], role, NOW)
            .await?;
        assert_eq!(
            store.team(&manager, &team, NOW).await?.caller_role,
            expected
        );
    }
    assert!(matches!(
        store
            .set_member_role(&manager, &team, &users[2], MembershipRole::Manager, NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    assert!(matches!(
        store
            .remove_team_member(&manager, &team, &users[0], NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    assert!(matches!(
        store
            .remove_team_member(&owner, &team, &users[1], NOW)
            .await,
        Err(StoreError::Forbidden)
    ));
    store
        .update_team(
            &manager,
            &team,
            &TeamPatch {
                name: Some("Updated".to_owned()),
            },
            NOW,
        )
        .await?;
    let updated = store.team(&member, &team, NOW).await?;
    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.handle.as_deref(), Some("team-a"));
    assert!(matches!(
        store
            .invite_member(
                &manager,
                &InviteMember {
                    team_id: team.clone(),
                    recipient: EmailAddress::parse("user2@example.invalid")?,
                    signup: None
                },
                NOW
            )
            .await,
        Err(StoreError::AlreadyMember)
    ));

    let unknown = store
        .invite_member(
            &manager,
            &InviteMember {
                team_id: team.clone(),
                recipient: EmailAddress::parse("unknown@example.invalid")?,
                signup: None,
            },
            NOW,
        )
        .await?;
    assert!(unknown.issued_signup_grant_id.is_none());
    let unknown_mail = store
        .invite_member(
            &manager,
            &InviteMember {
                team_id: team.clone(),
                recipient: EmailAddress::parse("mail@example.invalid")?,
                signup: Some(TeamSignupGrant {
                    proof: BearerSecret::generate()?.digest(),
                    expires_at: NOW + 1000,
                }),
            },
            NOW,
        )
        .await?;
    assert!(unknown_mail.issued_signup_grant_id.is_some());
    // Fault after grant insertion proves the whole accepting transaction rolls
    // back; callers no longer choose entity IDs to provoke a collision.
    let install = if postgres {
        "CREATE FUNCTION fail_invitation() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'Synthetic invitation failure'; END $$; CREATE TRIGGER fail_invitation BEFORE INSERT ON invitations FOR EACH ROW WHEN (NEW.recipient_email_key='rollback@example.invalid') EXECUTE FUNCTION fail_invitation()"
    } else {
        "CREATE TRIGGER fail_invitation BEFORE INSERT ON invitations WHEN NEW.recipient_email_key='rollback@example.invalid' BEGIN SELECT RAISE(ABORT, 'Synthetic invitation failure'); END"
    };
    tokio::task::block_in_place(|| connection.batch_execute(install))?;
    let failed = store
        .invite_member(
            &manager,
            &InviteMember {
                team_id: team.clone(),
                recipient: EmailAddress::parse("rollback@example.invalid")?,
                signup: Some(TeamSignupGrant {
                    proof: BearerSecret::generate()?.digest(),
                    expires_at: NOW + 1000,
                }),
            },
            NOW,
        )
        .await;
    let cleanup = tokio::task::block_in_place(|| {
        connection.batch_execute(if postgres {
            "DROP TRIGGER fail_invitation ON invitations; DROP FUNCTION fail_invitation()"
        } else {
            "DROP TRIGGER fail_invitation"
        })
    });
    assert!(matches!(failed, Err(StoreError::Database(_))));
    cleanup?;
    assert_eq!(store.team_invitations(&manager, &team, NOW).await?.len(), 4);
    assert!(matches!(
        store.team_invitations(&member, &team, NOW).await,
        Err(StoreError::Forbidden)
    ));
    let decline = store
        .invite_member(
            &owner,
            &InviteMember {
                team_id: team.clone(),
                recipient: EmailAddress::parse("user3@example.invalid")?,
                signup: Some(TeamSignupGrant {
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
            .process_invitation(&stranger, &decline.invitation.id, false, NOW)
            .await?
            .status,
        InvitationStatus::Declined
    );
    assert!(matches!(
        store.team(&stranger, &team, NOW).await,
        Err(StoreError::NotFound)
    ));

    let race = store
        .invite_member(
            &owner,
            &InviteMember {
                team_id: team.clone(),
                recipient: EmailAddress::parse("user3@example.invalid")?,
                signup: None,
            },
            NOW,
        )
        .await?
        .invitation
        .id;
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let first_store = store.clone();
    let second_store = store.clone();
    let first_barrier = barrier.clone();
    let second_barrier = barrier;
    let first_digest = tokens[3].digest();
    let second_digest = tokens[3].digest();
    let first_id = race.clone();
    let second_id = race;
    let first = tokio::spawn(async move {
        first_barrier.wait().await;
        first_store
            .process_invitation(&first_digest, &first_id, true, NOW)
            .await
    });
    let second = tokio::spawn(async move {
        second_barrier.wait().await;
        second_store
            .process_invitation(&second_digest, &second_id, true, NOW)
            .await
    });
    let (first, second) = tasks::join_pair(first, second, None).await?;
    let results = [first, second];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert!(
        results
            .iter()
            .filter_map(|result| result.as_ref().err())
            .all(|error| matches!(error, StoreError::AlreadyProcessed))
    );
    assert_eq!(store.team_members(&owner, &team, NOW).await?.len(), 4);
    store
        .remove_team_member(&manager, &team, &users[3], NOW)
        .await?;
    assert!(matches!(
        store.team(&stranger, &team, NOW).await,
        Err(StoreError::NotFound)
    ));
    store.leave_team(&member, &team, NOW).await?;
    assert!(matches!(
        store.team(&member, &team, NOW).await,
        Err(StoreError::NotFound)
    ));
    let rejoin = store
        .invite_member(
            &owner,
            &InviteMember {
                team_id: team.clone(),
                recipient: EmailAddress::parse("user2@example.invalid")?,
                signup: None,
            },
            NOW,
        )
        .await?
        .invitation
        .id;
    store
        .process_invitation(&member, &rejoin, true, NOW)
        .await?;
    let transfer = store.transfer_team(&owner, &team, &users[2], NOW).await?;
    assert!(!store.current_identity(&owner, NOW).await?.is_admin);
    assert!(!store.current_identity(&member, NOW).await?.is_admin);
    assert!(store.current_identity(&stranger, NOW).await?.is_admin);
    assert_eq!(transfer.owner_user_id, users[2]);
    assert_eq!(transfer.caller_role, TeamRole::Manager);
    assert_eq!(
        store.team(&member, &team, NOW).await?.caller_role,
        TeamRole::Owner
    );
    assert!(matches!(
        store.delete_team(&owner, &team, NOW).await,
        Err(StoreError::Forbidden)
    ));
    store.leave_team(&owner, &team, NOW).await?;
    assert_eq!(
        store.team(&owner, &personals[0], NOW).await?.kind,
        TeamKind::Personal
    );
    store.leave_team(&manager, &team, NOW).await?;
    store.delete_team(&member, &team, NOW).await?;
    assert!(matches!(
        store.team(&member, &team, NOW).await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store.team(&member, &personals[2], NOW).await?.kind,
        TeamKind::Personal
    );

    let race_team = store
        .create_team(
            &owner,
            &CreateTeam {
                name: "Race".to_owned(),
                handle: "transfer-race".to_owned(),
            },
            NOW,
        )
        .await?
        .id;
    let invite = store
        .invite_member(
            &owner,
            &InviteMember {
                team_id: race_team.clone(),
                recipient: EmailAddress::parse("user3@example.invalid")?,
                signup: None,
            },
            NOW,
        )
        .await?
        .invitation
        .id;
    store
        .process_invitation(&stranger, &invite, true, NOW)
        .await?;
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let transferring = store.clone();
    let leaving = store.clone();
    let transfer_barrier = barrier.clone();
    let leave_barrier = barrier;
    let owner_digest = tokens[0].digest();
    let target_digest = tokens[3].digest();
    let transfer_team = race_team.clone();
    let leave_team = race_team.clone();
    let target_user = users[3].clone();
    let transfer = tokio::spawn(async move {
        transfer_barrier.wait().await;
        transferring
            .transfer_team(&owner_digest, &transfer_team, &target_user, NOW)
            .await
    });
    let leave = tokio::spawn(async move {
        leave_barrier.wait().await;
        leaving.leave_team(&target_digest, &leave_team, NOW).await
    });
    match tasks::join_pair(transfer, leave, None).await? {
        (Ok(summary), Err(StoreError::OwnerTransferRequired)) => {
            assert_eq!(summary.owner_user_id, users[3]);
            assert_eq!(
                store.team(&stranger, &race_team, NOW).await?.caller_role,
                TeamRole::Owner
            );
            store.delete_team(&stranger, &race_team, NOW).await?;
        }
        (Err(StoreError::NotFound), Ok(())) => {
            assert_eq!(
                store.team(&owner, &race_team, NOW).await?.owner_user_id,
                users[0]
            );
            store.delete_team(&owner, &race_team, NOW).await?;
        }
        outcome => return Err(format!("Unexpected transfer/leave ordering: {outcome:?}").into()),
    }
    Ok(())
}

async fn collection_retry(
    store: &Store,
    tokens: &[BearerSecret],
    connection: &mut impl SimpleConnection,
    postgres: bool,
) -> Result<(), TestError> {
    let owner = tokens[0].digest();
    let stranger = tokens[3].digest();
    let create = CreateTeam {
        name: "Collection retry".to_owned(),
        handle: "collection-retry".to_owned(),
    };
    let original = store.create_team(&owner, &create, NOW).await?.id;
    let install = if postgres {
        "CREATE FUNCTION fail_collection() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'Synthetic collection failure'; END $$; CREATE TRIGGER fail_collection BEFORE DELETE ON teams FOR EACH ROW WHEN (OLD.handle='collection-retry') EXECUTE FUNCTION fail_collection()"
    } else {
        "CREATE TRIGGER fail_collection BEFORE DELETE ON teams WHEN OLD.handle='collection-retry' BEGIN SELECT RAISE(ABORT, 'Synthetic collection failure'); END"
    };
    tokio::task::block_in_place(|| connection.batch_execute(install))?;
    assert!(matches!(
        store.delete_team(&owner, &original, NOW).await,
        Err(StoreError::Database(_))
    ));
    assert!(matches!(
        store.team(&owner, &original, NOW).await,
        Err(StoreError::NotFound)
    ));
    assert!(
        store
            .teams(&owner, NOW)
            .await?
            .iter()
            .all(|team| team.id != original)
    );
    let reused = store.create_team(&owner, &create, NOW + 1).await?;
    assert_ne!(original, reused.id);
    assert!(matches!(
        store.delete_team(&stranger, &original, NOW + 1).await,
        Err(StoreError::NotFound)
    ));
    tokio::task::block_in_place(|| {
        connection.batch_execute(if postgres {
            "DROP TRIGGER fail_collection ON teams; DROP FUNCTION fail_collection()"
        } else {
            "DROP TRIGGER fail_collection"
        })
    })?;
    store.delete_team(&owner, &original, NOW + 2).await?;
    assert!(matches!(
        store.delete_team(&owner, &original, NOW + 3).await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(
        store
            .team(&owner, &reused.id, NOW + 3)
            .await?
            .handle
            .as_deref(),
        Some("collection-retry")
    );
    store.delete_team(&owner, &reused.id, NOW + 3).await?;
    Ok(())
}

macro_rules! verify_final {
    ($connection:expr) => {{
        assert_eq!(
            diesel::sql_query(
                "SELECT count(*) AS n FROM grants WHERE email_key='rollback@example.invalid'"
            )
            .get_result::<Count>($connection)?
            .n,
            0
        );
        assert_eq!(
            diesel::sql_query("SELECT count(*) AS n FROM users")
                .get_result::<Count>($connection)?
                .n,
            4
        );
        assert_eq!(
            diesel::sql_query("SELECT count(*) AS n FROM personal_contexts")
                .get_result::<Count>($connection)?
                .n,
            4
        );
        assert_eq!(
            diesel::sql_query("SELECT count(*) AS n FROM memberships")
                .get_result::<Count>($connection)?
                .n,
            5
        );
        assert_eq!(
            diesel::sql_query("SELECT count(*) AS n FROM invitations")
                .get_result::<Count>($connection)?
                .n,
            0
        );
        assert_eq!(
            diesel::sql_query("SELECT count(*) AS n FROM teams")
                .get_result::<Count>($connection)?
                .n,
            5
        );
    }};
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
    tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path.clone(), cipher).await?;
        seed(&store, &tokens).await?;
        collection_retry(&store, &tokens, &mut connection, false).await?;
        exercise(&store, &tokens, &mut connection, false).await
    })?;
    verify_final!(&mut connection);
    Ok(())
}

#[test]
fn team_and_invitation_operations_on_postgres() -> Result<(), TestError> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let unique = BearerSecret::generate()?;
    let schema = format!("team_test_{}", hex(&unique.digest().as_bytes()[..8]));
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
        tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_postgres(url.clone(), cipher).await?;
            seed(&store, &tokens).await?;
            collection_retry(&store, &tokens, &mut connection, true).await?;
            exercise(&store, &tokens, &mut connection, true).await
        })?;
        verify_final!(&mut connection);
        Ok(())
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
