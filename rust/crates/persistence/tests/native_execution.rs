//! Both-engine native reservation/evidence and conservative retirement admission.
use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, RootKey};
use chapchoo_domain::{
    DockerContainerId, DockerDaemonId, DockerVolumeIdentity, LifecycleAction, LifecyclePhase,
    NativeIdentity, NativeReservation, NativeScope, ResourcePresence,
};
use chapchoo_persistence::{
    ComputerRequest, ExecutionDraft, ExecutionResources, LifecycleRequest, ManagedResourceKind,
    ProviderSource, ResourceEvidence, ResourceReport, Store, StoreError, prepare_postgres,
    prepare_sqlite,
};
use diesel::{Connection, connection::SimpleConnection};
use diesel_async::{AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

type Error = Box<dyn std::error::Error + Send + Sync>;

fn now() -> Result<i64, Error> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .try_into()?)
}

fn cipher() -> Result<Arc<CredentialCipher>, Error> {
    Ok(Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([11; 32]),
    )?)?))
}

fn seed(
    cipher: &CredentialCipher,
    user: &BearerSecret,
    provider: &BearerSecret,
    postgres: bool,
) -> Result<String, Error> {
    let binary = |bytes: &[u8]| {
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        if postgres {
            format!("decode('{hex}','hex')")
        } else {
            format!("X'{hex}'")
        }
    };
    let session = cipher.protect_latest("session", user)?;
    let key = cipher.protect_latest("computer-provider:docker", provider)?;
    let hash = binary(&[8; 32]);
    let expiry = now()? + 86_400_000;
    Ok(format!("BEGIN;
        INSERT INTO users(id,email_key,delivery_email,password_hash,created_at,language,time_zone,region) VALUES('user','native@example.invalid','native@example.invalid','synthetic',0,'en','UTC','en-US');
        INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('personal','PERSONAL','Personal','personal','user',0);
        INSERT INTO memberships VALUES('personal','user','MEMBER');
        INSERT INTO personal_contexts VALUES('user','personal');
        INSERT INTO teams(id,kind,name,handle,owner_user_id,created_at) VALUES('scope','COLLABORATIVE','Scope','scope','user',0);
        INSERT INTO memberships VALUES('scope','user','MANAGER');
        INSERT INTO auth_sessions(id,user_id,current_hash,recovery_nonce,recovery_ciphertext,created_at,rotated_at,expires_at) VALUES('session','user',{},{},{},0,0,{expiry});
        INSERT INTO computer_providers(id,owner_kind,system_owner_id,name,provider_kind,docker_daemon_id,can_create,can_start,can_stop,can_restart,created_at) VALUES('docker','SYSTEM','system','Docker','DOCKER','daemon:original',TRUE,TRUE,TRUE,TRUE,0);
        INSERT INTO provider_keys(provider_id,current_hash,issued_at,expires_at,current_request_id,recovery_nonce,recovery_ciphertext) VALUES('docker',{},0,{expiry},'key',{},{});
        INSERT INTO control_instances(id,endpoint,renewed_at,expires_at) VALUES('control','http://control.invalid:1',0,{expiry});
        INSERT INTO provider_connections(provider_id,control_id,connection_id,credential_hash,expires_at) VALUES('docker','control','route',{},{expiry});
        INSERT INTO computers(id,owner_user_id,kind,provider_id,desired_state,current_request_id,current_payload_hash,current_action,current_phase,created_at) VALUES('computer','user','CLOUD','docker','RUNNING','create',{hash},'CREATE','ACCEPTED',0);
        INSERT INTO agents(id,owner_user_id,team_id,name,computer_id,computer_kind,created_at) VALUES('agent','user','scope','Native','computer','CLOUD',0);
        COMMIT;",
        binary(user.digest().as_bytes()), binary(session.nonce()), binary(session.ciphertext()),
        binary(provider.digest().as_bytes()), binary(key.nonce()), binary(key.ciphertext()),
        binary(provider.digest().as_bytes()),
    ))
}

fn draft(id: &str, daemon: &str) -> Result<ExecutionDraft, Error> {
    Ok(ExecutionDraft {
        id: id.into(),
        native: ExecutionResources::Docker {
            daemon: DockerDaemonId::parse(daemon.into())?,
            container_name: format!("container-{id}"),
            saved_volume_name: format!("volume-{id}"),
        },
    })
}

async fn exercise(
    store: Store,
    user: BearerSecret,
    provider: BearerSecret,
    scoped_delete: bool,
) -> Result<(), Error> {
    let digest = provider.digest();
    let source = ProviderSource {
        provider_id: "docker",
        control_id: "control",
        connection_id: "route",
        credential: &digest,
    };
    let scope = ComputerRequest {
        computer_id: "computer",
        request_id: "create",
    };
    assert!(matches!(
        store
            .reserve_execution(&source, scope, &draft("wrong", "daemon:other")?)
            .await,
        Err(StoreError::Conflict)
    ));
    let reserved = store
        .reserve_execution(&source, scope, &draft("first", "daemon:original")?)
        .await?;
    assert_eq!(reserved.resources.len(), 2);
    assert!(
        reserved
            .resources
            .iter()
            .all(|r| r.identity().is_none() && r.kubernetes_uid().is_none())
    );
    let saved = reserved
        .resources
        .iter()
        .find(|r| r.kind() == ManagedResourceKind::DockerVolume)
        .ok_or("Missing saved volume")?
        .native()
        .clone();
    let container = reserved
        .resources
        .iter()
        .find(|r| r.kind() == ManagedResourceKind::DockerContainer)
        .ok_or("Missing container")?
        .native()
        .clone();
    assert!(
        store
            .claim_execution_dispatch(&source, scope, "first")
            .await?
    );
    store.mark_start_dispatched(&source, scope, "first").await?;
    let volume = NativeIdentity::DockerVolume(DockerVolumeIdentity::local(
        "local",
        "local",
        &BTreeMap::new(),
        Some("2026-11-03T00:00:00Z".into()),
        BTreeMap::from([
            ("provider".into(), "docker".into()),
            ("computer".into(), "computer".into()),
        ]),
    )?);
    let full_id = NativeIdentity::DockerContainer(DockerContainerId::parse("a".repeat(64))?);
    for (native, execution, identity) in [
        (&saved, None, &volume),
        (&container, Some("first"), &full_id),
    ] {
        store
            .report_resource(
                &source,
                scope,
                ResourceReport {
                    native,
                    execution_id: execution,
                    evidence: ResourceEvidence::OwnedPresent { identity },
                },
            )
            .await?;
    }
    store
        .report_execution_running(&source, scope, "first")
        .await?;
    let foreign = NativeIdentity::DockerContainer(DockerContainerId::parse("b".repeat(64))?);
    assert!(matches!(
        store
            .report_resource(
                &source,
                scope,
                ResourceReport {
                    native: &container,
                    execution_id: Some("first"),
                    evidence: ResourceEvidence::OwnedPresent { identity: &foreign },
                }
            )
            .await,
        Err(StoreError::Conflict)
    ));
    let wrong_daemon = NativeReservation::new(
        ManagedResourceKind::DockerVolume,
        NativeScope::DockerDaemon(DockerDaemonId::parse("daemon:other".into())?),
        saved.name().into(),
    )?;
    assert!(matches!(
        store
            .report_resource(
                &source,
                scope,
                ResourceReport {
                    native: &wrong_daemon,
                    execution_id: None,
                    evidence: ResourceEvidence::Unknown,
                }
            )
            .await,
        Err(StoreError::Conflict)
    ));
    assert!(
        store
            .claim_native_retirement(&source, scope, None)
            .await
            .is_err(),
        "Ordinary running intent cannot claim retirement"
    );
    let admission = store
        .claim_native_retirement(&source, scope, Some("first"))
        .await?
        .ok_or("Terminal claim absent")?;
    assert_eq!(admission.previous_phase(), "COMPLETED");
    assert!(
        store
            .claim_native_retirement(&source, scope, Some("first"))
            .await?
            .is_none()
    );
    assert!(
        store
            .report_execution_running(&source, scope, "first")
            .await
            .is_err(),
        "A late Running observation cannot clear admitted native retirement"
    );
    // Exact known NotStarted release remains available to the same live source.
    store
        .release_unstarted_native_retirement(&source, &admission)
        .await?;
    store
        .report_execution_running(&source, scope, "first")
        .await?;
    let _accepted = store
        .claim_native_retirement(&source, scope, Some("first"))
        .await?
        .ok_or("Missing admission")?;
    // A new Delete must retain uncertainty from possibly accepted prior effects.
    let delete_request = if scoped_delete {
        store.delete_team(&user.digest(), "scope", now()?).await?;
        let current = store
            .managed_computers(&source, None, 20)
            .await?
            .into_iter()
            .find(|c| c.id == "computer")
            .ok_or("Missing scoped cleanup")?;
        assert_eq!(current.current_phase, "UNCERTAIN");
        current.current_request_id
    } else {
        let deleted = store
            .request_lifecycle(
                &user.digest(),
                "agent",
                &LifecycleRequest {
                    request_id: "delete".into(),
                    expected_request_id: "create".into(),
                    action: LifecycleAction::Delete,
                },
                now()?,
            )
            .await?;
        assert_eq!(deleted.computer.progress.phase, LifecyclePhase::Uncertain);
        "delete".into()
    };
    let deleting = ComputerRequest {
        computer_id: "computer",
        request_id: &delete_request,
    };
    assert!(
        store
            .claim_native_retirement(&source, deleting, None)
            .await?
            .is_none()
    );
    assert!(
        store
            .release_unstarted_native_retirement(&source, &admission)
            .await
            .is_err(),
        "An old request cannot release the new Delete"
    );
    store
        .report_resource(
            &source,
            deleting,
            ResourceReport {
                native: &container,
                execution_id: Some("first"),
                evidence: ResourceEvidence::DefinitivelyAbsent {
                    expected_identity: Some(&full_id),
                },
            },
        )
        .await?;
    store.retire_execution(&source, deleting, "first").await?;
    assert!(
        store
            .confirm_computer_cleanup(&source, deleting)
            .await
            .is_err(),
        "Subset absence is not complete cleanup or saved-volume suffix replay authority"
    );
    store
        .report_resource(
            &source,
            deleting,
            ResourceReport {
                native: &saved,
                execution_id: None,
                evidence: ResourceEvidence::Unknown,
            },
        )
        .await?;
    let original = store.managed_resources(&source, deleting).await?;
    assert_eq!(
        original
            .iter()
            .find(|r| r.kind() == ManagedResourceKind::DockerVolume)
            .ok_or("Missing original volume")?
            .identity(),
        Some(&volume)
    );
    store
        .report_resource(
            &source,
            deleting,
            ResourceReport {
                native: &saved,
                execution_id: None,
                evidence: ResourceEvidence::DefinitivelyAbsent {
                    expected_identity: Some(&volume),
                },
            },
        )
        .await?;
    assert!(
        matches!(
            store
                .report_resource(
                    &source,
                    deleting,
                    ResourceReport {
                        native: &saved,
                        execution_id: None,
                        evidence: ResourceEvidence::OwnedPresent { identity: &volume },
                    }
                )
                .await,
            Err(StoreError::Conflict)
        ),
        "Original disappeared storage cannot be recreated"
    );
    assert!(
        store
            .managed_resources(&source, deleting)
            .await?
            .iter()
            .all(|r| r.presence == ResourcePresence::Absent)
    );
    store.confirm_computer_cleanup(&source, deleting).await?;
    Ok(())
}

#[test]
fn sqlite_native_execution_preserves_original_storage_and_uncertain_retirement() -> Result<(), Error>
{
    sqlite_run(false)
}

#[test]
fn sqlite_scoped_deletion_cannot_clear_native_retirement_uncertainty() -> Result<(), Error> {
    sqlite_run(true)
}

fn sqlite_run(scoped_delete: bool) -> Result<(), Error> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("native.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let user = BearerSecret::generate()?;
    let provider = BearerSecret::generate()?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    connection.batch_execute("PRAGMA foreign_keys=ON")?;
    connection.batch_execute(&seed(&cipher, &user, &provider, false)?)?;
    tokio::runtime::Runtime::new()?.block_on(async {
        exercise(
            Store::connect_sqlite(path, cipher).await?,
            user,
            provider,
            scoped_delete,
        )
        .await
    })
}

#[test]
fn postgres_native_execution_preserves_original_storage_and_uncertain_retirement()
-> Result<(), Error> {
    // Explicit preparation holds a server-wide advisory lock, even for isolated
    // schemas. Run the two owned scenarios in order instead of racing that gate.
    postgres_run(false)?;
    postgres_run(true)
}

fn postgres_run(scoped_delete: bool) -> Result<(), Error> {
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut admin = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("native_execution_{suffix}");
    admin.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let user = BearerSecret::generate()?;
        let provider = BearerSecret::generate()?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        connection.batch_execute(&seed(&cipher, &user, &provider, true)?)?;
        tokio::runtime::Runtime::new()?.block_on(async {
            exercise(
                Store::connect_postgres(url, cipher).await?,
                user,
                provider,
                scoped_delete,
            )
            .await
        })
    })();
    let cleanup = admin.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    result?;
    cleanup?;
    Ok(())
}
