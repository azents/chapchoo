//! Repository acceptance, profile editing and exact Team-scoped identity.
use super::*;
use crate::{
    ComponentKeyDraft, ComponentKeyPolicy, CreateTeam, DirectoryPolicy, GrantDraft, NewAccount,
    NewSession, ProviderOwner, SessionPolicy,
};
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{
    BearerSecret, BootstrapProviderPermit, CredentialCipher, PasswordCost, PasswordHasher, RootKey,
};
use chapchoo_domain::{EmailAddress, Password};
use diesel::{Connection, connection::SimpleConnection};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::sync::Arc;

type TestError = Box<dyn std::error::Error + Send + Sync>;

async fn authentication(
    store: &Store,
    actor: &SecretDigest,
    request: &str,
    now: i64,
) -> Result<String, TestError> {
    match store
        .register_external_api_key(
            actor,
            &crate::RegisterApiKey {
                scope: crate::AuthenticationScope::Personal,
                request_id: request.to_owned(),
                name: "Fixture authentication".into(),
                key: chapchoo_crypto::ExternalSecret::new(b"synthetic-profile-api-key".to_vec())?,
            },
            now,
        )
        .await?
    {
        crate::AuthenticationAcceptedResult::Record { authentication_id } => Ok(authentication_id),
        _ => Err("Missing profile fixture authentication".into()),
    }
}

fn cipher() -> Result<Arc<CredentialCipher>, TestError> {
    Ok(Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([64; 32]),
    )?)?))
}
fn account(email: &str) -> Result<NewAccount, TestError> {
    Ok(NewAccount {
        email: EmailAddress::parse(email)?,
        password: PasswordHasher::new(PasswordCost::new(10)?)
            .hash(&Password::parse("Aa1!profiles".into())?)?,
        session: NewSession {
            token: BearerSecret::generate()?,
        },
        display: None,
    })
}
fn present(outcome: CreationOutcome) -> Result<AgentSummary, TestError> {
    match outcome {
        CreationOutcome::Present(agent) => Ok(*agent),
        CreationOutcome::Removed { .. } => Err("Expected a present Agent".into()),
    }
}

async fn exercise(store: Store) -> Result<(), TestError> {
    let now = operation!(store, |connection, dialect| {
        Ok(crate::directory::database_now!(connection, dialect))
    })?;
    let setup = BearerSecret::generate()?;
    let admin = store
        .initialize_admin(
            &setup.digest(),
            &setup,
            &account("profiles@example.invalid")?,
            now,
            SessionPolicy::default(),
        )
        .await?;
    let actor = admin.token.digest();
    let member_draft = account("other-profiles@example.invalid")?;
    let signup = BearerSecret::generate()?;
    store
        .issue_signup_grant(
            &actor,
            &member_draft.email,
            &GrantDraft {
                proof: signup.digest(),
                expires_at: now + 1000,
            },
            now,
        )
        .await?;
    let member = store
        .redeem_signup(
            &signup.digest(),
            &member_draft,
            now,
            SessionPolicy::default(),
        )
        .await?;
    let other_actor = member.token.digest();
    let personal = store.teams(&actor, now).await?.remove(0);
    let other_team = store.teams(&other_actor, now).await?.remove(0);
    let collaborative = store
        .create_team(
            &actor,
            &CreateTeam {
                name: "Profiles".into(),
                handle: "profiles".into(),
            },
            now,
        )
        .await?;
    let images = store.agent_profile_images(&actor, None, 100, now).await?;
    let first_page = store.agent_profile_images(&actor, None, 2, now).await?;
    let second_page = store
        .agent_profile_images(&actor, Some(&first_page[1].id), 2, now)
        .await?;
    assert_eq!(first_page[0].id, images[0].id);
    assert_eq!(second_page[0].id, images[2].id);
    assert!(
        store
            .agent_profile_images(&actor, Some(&images[5].id), 2, now)
            .await?
            .is_empty()
    );
    assert!(matches!(
        store.agent_profile_images(&actor, None, 101, now).await,
        Err(StoreError::InvalidInput)
    ));
    assert!(matches!(
        store
            .agent_profile_images(&actor, Some(&admin.user.id), 2, now)
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert_eq!(
        store.team_by_handle(&actor, "profiles", now).await?.id,
        collaborative.id
    );
    assert!(matches!(
        store.team_by_handle(&other_actor, "profiles", now).await,
        Err(StoreError::NotFound)
    ));
    assert!(
        !store
            .team_handle_available(&other_actor, "profiles", now)
            .await?
    );
    assert!(store.team_handle_available(&actor, "new-team", now).await?);
    assert!(matches!(
        store.team_by_handle(&actor, "my", now).await,
        Err(StoreError::InvalidInput)
    ));
    let proof = BootstrapProviderPermit::new("profile-provider".into())?.authenticated();
    let substrate = ProviderSubstrate::Kubernetes;
    let provider = store
        .rotate_provider_key(
            ProviderOwner::Bootstrap {
                proof: &proof,
                name: "Profiles",
                substrate: &substrate,
                capabilities: ProviderCapabilities {
                    create: true,
                    start: true,
                    stop: true,
                    restart: true,
                },
            },
            "profile-provider",
            &ComponentKeyDraft {
                request_id: format!("{now}:profiles"),
                token: BearerSecret::generate()?,
            },
            ComponentKeyPolicy::default(),
            now,
        )
        .await?;
    let control = store
        .register_control("http://127.0.0.1:1", DirectoryPolicy::default())
        .await?;
    let connection = store
        .prepare_provider_connection(
            &provider.provider_id,
            &provider.key.token.digest(),
            &control.id,
        )
        .await?;
    store
        .admit_provider_connection(
            &provider.provider_id,
            &provider.key.token.digest(),
            connection.expected.as_ref(),
            &connection.owner,
            DirectoryPolicy::default(),
        )
        .await?;
    let authentication_id =
        authentication(&store, &actor, "profile-fixture-authentication", now).await?;
    let other_authentication =
        authentication(&store, &other_actor, "profile-fixture-authentication", now).await?;
    let draft = CreateCloudAgent {
        team_id: personal.id.clone(),
        authentication_id,
        request_id: "create-profile".into(),
        provider_id: provider.provider_id.clone(),
        name: "First profile".into(),
        profile: CreateAgentProfile {
            handle: "first-agent".into(),
            image_id: images[0].id.clone(),
            palette_key: "sage".into(),
        },
    };
    assert!(
        store
            .agent_handle_available(&actor, &personal.id, &draft.profile.handle, now)
            .await?
    );
    let (first, recovered) = tokio::join!(
        store.create_cloud_agent(&actor, &draft, now),
        store.create_cloud_agent(&actor, &draft, now),
    );
    let first = present(first?)?;
    let recovered = present(recovered?)?;
    assert_eq!(first.id, recovered.id);
    assert_eq!(first.computer.id, recovered.computer.id);
    let saved = first
        .authentication
        .as_ref()
        .ok_or("Missing atomic creation selection")?;
    assert_eq!(saved.authentication_id, draft.authentication_id);
    assert_eq!(saved.request_id, draft.request_id);
    let mut missing_authentication = draft_copy(&draft);
    missing_authentication.request_id = "missing-authentication".into();
    missing_authentication.profile.handle = "missing-auth".into();
    missing_authentication.authentication_id.clear();
    assert!(matches!(
        store
            .create_cloud_agent(&actor, &missing_authentication, now)
            .await,
        Err(StoreError::InvalidInput)
    ));
    let mut foreign_authentication = draft_copy(&draft);
    foreign_authentication.request_id = "foreign-authentication".into();
    foreign_authentication.profile.handle = "foreign-auth".into();
    foreign_authentication.authentication_id = other_authentication.clone();
    assert!(matches!(
        store
            .create_cloud_agent(&actor, &foreign_authentication, now)
            .await,
        Err(StoreError::NotFound)
    ));
    EntityId::from_dto(EntityKind::Agent, &first.id)?;
    EntityId::from_dto(EntityKind::Computer, &first.computer.id)?;
    assert_eq!(first.handle, draft.profile.handle);
    assert_eq!(first.image_id, draft.profile.image_id);
    assert_eq!(first.image_asset_key, images[0].asset_key);
    assert_eq!(first.palette_key, "sage");
    assert!(
        store
            .resource_inventory(&actor, &first.computer.id, None, 20, now)
            .await?
            .is_empty()
    );
    let provider_key = provider.key.token.digest();
    let source = crate::ProviderSource {
        provider_id: &provider.provider_id,
        control_id: &control.id,
        connection_id: &connection.owner.connection_id,
        credential: &provider_key,
    };
    let scope = crate::ComputerRequest {
        computer_id: &first.computer.id,
        request_id: &draft.request_id,
    };
    let resources = |suffix: &str| crate::ExecutionDraft {
        native: crate::ExecutionResources::Kubernetes {
            namespace: "profiles".into(),
            pod_name: format!("pod-{suffix}"),
            bootstrap_secret_name: format!("bootstrap-{suffix}"),
            saved_pvc_name: format!("saved-{suffix}"),
        },
    };
    let reserved = store
        .reserve_execution(&source, scope, &resources("original"))
        .await?;
    EntityId::from_dto(EntityKind::Execution, &reserved.execution_id)?;
    assert_eq!(reserved.resources.len(), 3);
    for resource in &reserved.resources {
        assert_eq!(resource.allocation().computer(), first.computer.id);
        assert_eq!(resource.allocation().provider(), provider.provider_id);
        assert_eq!(
            resource.allocation().execution(),
            resource.execution_id.as_deref()
        );
    }
    let retry = store
        .reserve_execution(&source, scope, &resources("ignored"))
        .await?;
    assert_eq!(reserved.execution_id, retry.execution_id);
    assert_eq!(reserved.resources[0].native(), retry.resources[0].native());
    let managed = store.managed_computers(&source, None, 20).await?;
    assert_eq!(
        managed[0].current_execution_id.as_deref(),
        Some(reserved.execution_id.as_str())
    );
    assert!(
        store
            .managed_computers(&source, Some(&first.computer.id), 20)
            .await?
            .is_empty()
    );
    let snapshot = store
        .delivery_snapshot(Some(&actor), scope, Some(&reserved.execution_id))
        .await?;
    assert_eq!(snapshot.resources.len(), 3);
    assert!(matches!(
        store
            .delivery_snapshot(Some(&other_actor), scope, Some(&reserved.execution_id))
            .await,
        Err(StoreError::NotFound)
    ));
    let provider_snapshot = store
        .provider_delivery_snapshot(&source, scope, Some(&reserved.execution_id))
        .await?;
    assert_eq!(provider_snapshot.computer.id, first.computer.id);
    let connector = store
        .issue_connector_session_from_source(
            &source,
            &reserved.execution_id,
            &crate::ConnectorSessionDraft {
                request_id: "execution-bootstrap".into(),
                initial_key: BearerSecret::generate()?,
            },
            now,
        )
        .await?;
    store
        .verify_delivery_bootstrap(
            scope,
            &reserved.execution_id,
            &connector.id,
            &connector.initial_key.digest(),
        )
        .await?;
    assert!(
        !store
            .agent_handle_available(&other_actor, &other_team.id, &first.handle, now)
            .await?
    );
    assert_eq!(
        store
            .agent_in_team(&actor, &personal.id, &first.handle, now)
            .await?
            .id,
        first.id
    );
    assert!(matches!(
        store
            .agent_in_team(&actor, &collaborative.id, &first.handle, now)
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .agent_in_team(&other_actor, &personal.id, &first.handle, now)
            .await,
        Err(StoreError::NotFound)
    ));
    for field in 0..7 {
        let mut changed = draft_copy(&draft);
        match field {
            0 => changed.team_id = collaborative.id.clone(),
            1 => changed.name = "Changed name".into(),
            2 => changed.profile.handle = "changed-handle".into(),
            3 => changed.profile.image_id = images[1].id.clone(),
            4 => changed.profile.palette_key = "rose".into(),
            5 => changed.authentication_id = other_authentication.clone(),
            _ => changed.provider_id = EntityId::generate(now)?.to_dto(EntityKind::Provider),
        }
        assert!(matches!(
            store.create_cloud_agent(&actor, &changed, now).await,
            Err(StoreError::Conflict)
        ));
    }
    let mut changed = draft_copy(&draft);
    changed.request_id = "collision".into();
    changed.team_id = other_team.id.clone();
    changed.authentication_id = other_authentication.clone();
    assert!(matches!(
        store.create_cloud_agent(&other_actor, &changed, now).await,
        Err(StoreError::HandleConflict)
    ));
    changed = draft_copy(&draft);
    changed.request_id = "bad-image".into();
    changed.profile.handle = "bad-image".into();
    changed.profile.image_id = EntityId::generate(now)?.to_dto(EntityKind::ProfileImage);
    assert!(matches!(
        store.create_cloud_agent(&actor, &changed, now).await,
        Err(StoreError::InvalidInput)
    ));
    changed = draft_copy(&draft);
    changed.request_id = "manual-exact".into();
    changed.profile.handle = " First-agent".into();
    assert!(matches!(
        store.create_cloud_agent(&actor, &changed, now).await,
        Err(StoreError::InvalidInput)
    ));
    assert_eq!(
        store
            .computer_inventory(&actor, &provider.provider_id, None, 50, now)
            .await?
            .len(),
        1
    );

    let update = UpdateAgentProfile {
        name: "Saved profile".into(),
        image_id: images[1].id.clone(),
        palette_key: "rose".into(),
    };
    assert!(matches!(
        store
            .update_agent_profile(&other_actor, &first.id, &update, now)
            .await,
        Err(StoreError::NotFound)
    ));
    let saved = store
        .update_agent_profile(&actor, &first.id, &update, now)
        .await?;
    let identical = store
        .update_agent_profile(&actor, &first.id, &update, now)
        .await?;
    assert_eq!(saved.handle, first.handle);
    assert_eq!(saved.id, identical.id);
    assert_eq!(saved.computer.current_request_id, draft.request_id);
    assert_eq!(saved.image_id, images[1].id);
    assert_eq!(saved.image_asset_key, images[1].asset_key);
    assert_eq!(saved.palette_key, "rose");
    assert_eq!(
        store.agents(&actor, &personal.id, None, 20, now).await?[0].name,
        "Saved profile"
    );
    assert!(
        store
            .agents(&actor, &personal.id, Some(&first.id), 20, now)
            .await?
            .is_empty()
    );
    assert_eq!(
        present(store.create_cloud_agent(&actor, &draft, now).await?)?.name,
        "Saved profile"
    );

    store
        .request_lifecycle(
            &actor,
            &first.id,
            &LifecycleRequest {
                request_id: "delete-profile".into(),
                expected_request_id: draft.request_id.clone(),
                action: LifecycleAction::Delete,
            },
            now,
        )
        .await?;
    assert_eq!(
        store
            .force_remove_agent(&actor, &first.id, "delete-profile", now)
            .await?,
        AgentRemoval::Removed
    );
    assert_eq!(
        store
            .force_remove_agent(&actor, &first.id, "delete-profile", now)
            .await?,
        AgentRemoval::AlreadyRemoved
    );
    assert!(
        store
            .agent_handle_available(&actor, &personal.id, &first.handle, now)
            .await?
    );
    assert!(matches!(
        store
            .delete_external_authentication(
                &actor,
                &crate::DeleteAuthentication {
                    scope: crate::AuthenticationScope::Personal,
                    request_id: "delete-original-authentication".into(),
                    authentication_id: draft.authentication_id.clone(),
                    expected_request_id: "profile-fixture-authentication".into(),
                },
                now
            )
            .await?,
        crate::DeleteAuthenticationOutcome::Accepted(_)
    ));
    // Historical accepted payload recovery does not need a live credential.
    match store.create_cloud_agent(&actor, &draft, now).await? {
        CreationOutcome::Removed { assigned_agent_id } => assert_eq!(assigned_agent_id, first.id),
        CreationOutcome::Present(_) => return Err("Removed receipt allocated an Agent".into()),
    }
    changed = draft_copy(&draft);
    changed.request_id = "reuse-handle".into();
    let replacement_authentication =
        authentication(&store, &actor, "replacement-authentication", now).await?;
    changed.authentication_id = replacement_authentication.clone();
    let replacement = present(store.create_cloud_agent(&actor, &changed, now).await?)?;
    assert_ne!(replacement.id, first.id);
    assert_eq!(replacement.handle, first.handle);
    let mut left = draft_copy(&draft);
    left.authentication_id = replacement_authentication;
    left.request_id = "race-left".into();
    left.profile.handle = "racing-handle".into();
    let mut right = draft_copy(&left);
    right.request_id = "race-right".into();
    right.team_id = other_team.id.clone();
    right.authentication_id = other_authentication;
    let (left, right) = tokio::join!(
        store.create_cloud_agent(&actor, &left, now),
        store.create_cloud_agent(&other_actor, &right, now),
    );
    match (left, right) {
        (Ok(CreationOutcome::Present(_)), Err(StoreError::HandleConflict))
        | (Err(StoreError::HandleConflict), Ok(CreationOutcome::Present(_))) => {}
        _ => return Err("Global handle race did not produce exactly one winner".into()),
    }
    let race_authentication = authentication(&store, &actor, "race-authentication", now).await?;
    let mut racing_creation = draft_copy(&draft);
    racing_creation.request_id = "create-auth-race".into();
    racing_creation.profile.handle = "auth-race-agent".into();
    racing_creation.authentication_id = race_authentication.clone();
    let deletion = crate::DeleteAuthentication {
        scope: crate::AuthenticationScope::Personal,
        request_id: "delete-auth-race".into(),
        authentication_id: race_authentication.clone(),
        expected_request_id: "race-authentication".into(),
    };
    let (created, deleted) = tokio::join!(
        store.create_cloud_agent(&actor, &racing_creation, now),
        store.delete_external_authentication(&actor, &deletion, now)
    );
    match (created, deleted) {
        (Ok(CreationOutcome::Present(agent)), Ok(crate::DeleteAuthenticationOutcome::InUse)) => {
            assert_eq!(
                agent
                    .authentication
                    .ok_or("Missing raced selection")?
                    .authentication_id,
                race_authentication
            );
        }
        (
            Err(StoreError::NotFound),
            Ok(crate::DeleteAuthenticationOutcome::Accepted(
                crate::AuthenticationAcceptedResult::Deleted { authentication_id },
            )),
        ) => {
            assert_eq!(authentication_id, race_authentication);
            assert!(
                store
                    .agent_creation_result(&actor, &racing_creation.request_id, now)
                    .await?
                    .is_none()
            );
        }
        _ => return Err("Creation/deletion race violated atomic reference admission".into()),
    }
    store.drain_control(&control.id).await?;
    store
        .withdraw_drained_control_connections(&control.id)
        .await?;
    Ok(())
}

fn draft_copy(draft: &CreateCloudAgent) -> CreateCloudAgent {
    CreateCloudAgent {
        team_id: draft.team_id.clone(),
        authentication_id: draft.authentication_id.clone(),
        request_id: draft.request_id.clone(),
        provider_id: draft.provider_id.clone(),
        name: draft.name.clone(),
        profile: draft.profile.clone(),
    }
}

#[test]
fn sqlite_agent_acceptance_profiles_and_scoped_lookup() -> Result<(), TestError> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("profiles.sqlite");
    let cipher = cipher()?;
    crate::prepare_sqlite(&path, &cipher)?;
    tokio::runtime::Runtime::new()?
        .block_on(async { exercise(Store::connect_sqlite(path, cipher).await?).await })
}

#[test]
fn postgres_agent_acceptance_profiles_and_scoped_lookup() -> Result<(), TestError> {
    let _guard = crate::preparation::POSTGRES_PREPARATION_TEST_LOCK
        .lock()
        .map_err(|_| "PostgreSQL preparation lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let schema = format!("agent_profiles_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        crate::prepare_postgres(&url, &cipher)?;
        tokio::runtime::Runtime::new()?
            .block_on(async { exercise(Store::connect_postgres(url, cipher).await?).await })
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    match (result, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(primary), Ok(())) => Err(primary),
        (Ok(()), Err(cleanup)) => Err(cleanup.into()),
        (Err(primary), Err(cleanup)) => {
            Err(format!("{primary}; cleanup also failed: {cleanup}").into())
        }
    }
}
