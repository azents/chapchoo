mod support;

use base64::{Engine, engine::general_purpose::URL_SAFE};
use chapchoo_crypto::{BearerSecret, CredentialCipher, PasswordCost, RootKey};
use chapchoo_domain::{EmailAddress, Password};
use chapchoo_persistence::{Store, prepare_sqlite};
use chapchoo_protocol::authentication::{
    self as wire, authentications_client::AuthenticationsClient,
};
use chapchoo_rpc::{AuthenticationServices, memory_transport};
use chapchoo_service::{
    AcceptanceTime, AccountsConfig, AccountsService, AuthenticationService, PasswordWork,
    SessionPolicy,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tonic::{Code, Request, transport::Server};

type Error = Box<dyn std::error::Error + Send + Sync>;
fn auth<T>(value: T, token: &BearerSecret) -> Result<Request<T>, Error> {
    let mut request = Request::new(value);
    request.metadata_mut().insert(
        "authorization",
        format!("Bearer {}", token.expose()).parse()?,
    );
    Ok(request)
}
fn personal() -> Option<wire::Scope> {
    Some(wire::Scope {
        owner: Some(wire::scope::Owner::Personal(())),
    })
}
fn record(value: wire::AcceptedResult) -> Result<String, Error> {
    match value.result {
        Some(wire::accepted_result::Result::Record(value)) => Ok(value.authentication_id),
        _ => Err("Expected accepted record identity".into()),
    }
}
async fn due(value: wire::DeviceProgress) -> Result<String, Error> {
    let Some(wire::device_progress::State::Pending(pending)) = value.state else {
        return Err("Expected pending device authorization".into());
    };
    assert_eq!(
        pending.verification_uri,
        chapchoo_service::DEVICE_VERIFICATION_URI
    );
    assert_eq!(pending.user_code, "FIXTURE-CODE");
    let at = pending.next_check_at.ok_or("Missing next check time")?;
    let at = at
        .seconds
        .checked_mul(1000)
        .and_then(|s| s.checked_add(i64::from(at.nanos / 1_000_000)))
        .ok_or("Invalid next check time")?;
    // This elapsed-time wait verifies the provider's explicit polling cadence;
    // it is not synchronization for a background task or fixture readiness.
    let remaining = at.saturating_sub(AcceptanceTime::System.now()?);
    tokio::time::sleep(std::time::Duration::from_millis(u64::try_from(
        remaining.max(0),
    )?))
    .await;
    Ok(value.attempt_id)
}

#[test]
fn authentication_rpc_exposes_secret_free_catalogue_recovery_and_explicit_device_progress()
-> Result<(), Error> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("authentication.sqlite");
    let cipher = Arc::new(CredentialCipher::new(RootKey::parse(
        URL_SAFE.encode([83; 32]),
    )?)?);
    prepare_sqlite(&path, &cipher)?;
    let setup = BearerSecret::generate()?;
    tokio::runtime::Runtime::new()?.block_on(support::scope(async |fixture| {
        let store = Store::connect_sqlite(path, cipher).await?;
        let accounts = AccountsService::new(store.clone(), PasswordWork::new(1, PasswordCost::new(10)?)?, None, AccountsConfig {
            general_web_url: "http://localhost:3000".into(), setup_proof: Some(setup.digest()), session_policy: SessionPolicy::default(),
        })?;
        let actor = accounts.initialize(&setup, EmailAddress::parse("auth@example.invalid")?, Password::parse("Aa1!fixture-admin".into())?, AcceptanceTime::System, None).await?.token;
        let provider = Arc::new(support::authentication::Provider::default());
        let service = Arc::new(AuthenticationService::new(store, provider.clone(), 2)?);
        let admission = Arc::new(AtomicBool::new(true));
        let rpc = AuthenticationServices::new(service.clone()).with_admission(admission.clone());
        // This fixture owns Service completion even if an assertion/call fails.
        let stopped = fixture.shutdown();
        fixture.server(async move {
            let _ = stopped.await;
            service.close();
            service.finish().await
        });
        let (channel, incoming) = memory_transport();
        let stopped = fixture.shutdown();
        fixture.server(Server::builder().add_service(rpc.server()).serve_with_incoming_shutdown(incoming, async { let _ = stopped.await; }));
        let mut client = AuthenticationsClient::new(channel);
        assert_eq!(client.list(wire::ListRequest::default()).await.err().ok_or("Anonymous catalogue read")?.code(), Code::Unauthenticated);
        assert_eq!(client.list(auth(wire::ListRequest::default(), &actor)?).await.err().ok_or("Missing owner scope")?.code(), Code::InvalidArgument);
        let registration = wire::RegisterKeyRequest { scope: personal(), request_id: "register".into(), name: "Usage unverified".into(), key: b"fixture-api-key".to_vec() };
        assert!(!format!("{registration:?}").contains("fixture-api-key"));
        let id = record(client.register_key(auth(registration.clone(), &actor)?).await?.into_inner())?;
        assert_eq!(record(client.register_key(auth(registration, &actor)?).await?.into_inner())?, id);
        let item = client.get(auth(wire::RecordRequest { scope: personal(), authentication_id: id.clone() }, &actor)?).await?.into_inner();
        assert_eq!(item.kind, wire::Kind::OpenaiApiKey as i32);
        assert_eq!(item.current_request_id, "register");
        assert!(item.account_id.is_none() && item.email.is_none() && item.expires_at.is_none());
        let listed = client.list(auth(wire::ListRequest { scope: personal(), after: None, limit: 1 }, &actor)?).await?.into_inner();
        assert_eq!(listed.items.len(), 1);
        let end = client.list(auth(wire::ListRequest { scope: personal(), after: listed.next_after, limit: 1 }, &actor)?).await?.into_inner();
        assert!(end.items.is_empty());
        client.rename(auth(wire::RenameRequest { scope: personal(), request_id: "rename".into(), authentication_id: id.clone(), expected_request_id: "register".into(), name: "Renamed".into() }, &actor)?).await?;
        client.replace_key(auth(wire::ReplaceKeyRequest { scope: personal(), request_id: "replace".into(), authentication_id: id.clone(), expected_request_id: "rename".into(), key: b"fixture-new-key".to_vec() }, &actor)?).await?;
        let deleted = client.delete(auth(wire::DeleteRequest { scope: personal(), request_id: "delete".into(), authentication_id: id.clone(), expected_request_id: "replace".into() }, &actor)?).await?.into_inner();
        assert!(matches!(deleted.outcome, Some(wire::delete_result::Outcome::Accepted(_))));
        let historical = client.request_result(auth(wire::RequestResultRequest { request_id: "register".into() }, &actor)?).await?.into_inner();
        assert_eq!(historical.operation, wire::Operation::RegisterKey as i32);
        assert_eq!(record(historical.accepted.ok_or("Missing registration receipt")?)?, id);
        assert_eq!(provider.issues.load(Ordering::SeqCst), 0);
        let start = wire::StartDeviceRequest { scope: personal(), request_id: "device".into(), name: "ChatGPT".into(), target: None };
        let pending = client.start_device(auth(start.clone(), &actor)?).await?.into_inner();
        let attempt = pending.attempt_id.clone();
        let replay = client.start_device(auth(start, &actor)?).await?.into_inner();
        assert_eq!(replay.attempt_id, attempt);
        assert_eq!(provider.issues.load(Ordering::SeqCst), 1);
        let device = wire::DeviceRequest { attempt_id: attempt.clone() };
        client.device_status(auth(device.clone(), &actor)?).await?;
        assert_eq!(provider.polls.load(Ordering::SeqCst), 0);
        due(pending).await?;
        let completed = client.poll_device(auth(device.clone(), &actor)?).await?.into_inner();
        let Some(wire::device_progress::State::Registered(registered)) = completed.state else { return Err(Error::from("Device did not register directly")); };
        assert_eq!(provider.polls.load(Ordering::SeqCst), 1);
        assert_eq!(provider.exchanges.load(Ordering::SeqCst), 1);
        let completed_replay = client.poll_device(auth(device, &actor)?).await?.into_inner();
        assert!(matches!(completed_replay.state, Some(wire::device_progress::State::Registered(_))));
        assert_eq!(provider.exchanges.load(Ordering::SeqCst), 1);
        let item = client.get(auth(wire::RecordRequest { scope: personal(), authentication_id: registered.authentication_id.clone() }, &actor)?).await?.into_inner();
        assert_eq!(item.account_id.as_deref(), Some("fixture-account"));
        let reauth = client.start_device(auth(wire::StartDeviceRequest { scope: personal(), request_id: "reauth".into(), name: item.name, target: Some(wire::ReauthenticateTarget { authentication_id: item.id.clone(), expected_request_id: item.current_request_id }) }, &actor)?).await?.into_inner();
        let attempt_id = due(reauth).await?;
        let reauthenticated = client.poll_device(auth(wire::DeviceRequest { attempt_id }, &actor)?).await?.into_inner();
        assert!(matches!(reauthenticated.state, Some(wire::device_progress::State::Registered(value)) if value.authentication_id == item.id));
        let cancel = client.start_device(auth(wire::StartDeviceRequest { scope: personal(), request_id: "cancel-start".into(), name: "Cancelled".into(), target: None }, &actor)?).await?.into_inner();
        let cancelled = client.cancel_device(auth(wire::CancelDeviceRequest { attempt_id: cancel.attempt_id.clone(), request_id: "cancel".into() }, &actor)?).await?.into_inner();
        assert!(matches!(cancelled.state, Some(wire::device_progress::State::Cancelled(()))));
        client.poll_device(auth(wire::DeviceRequest { attempt_id: cancel.attempt_id }, &actor)?).await?;
        assert_eq!(provider.polls.load(Ordering::SeqCst), 2);
        admission.store(false, Ordering::Release);
        assert_eq!(client.list(auth(wire::ListRequest { scope: personal(), ..Default::default() }, &actor)?).await.err().ok_or("Closed admission accepted read")?.code(), Code::Unavailable);
        admission.store(true, Ordering::Release);
        accounts.logout(&actor.digest(), AcceptanceTime::System.now()?).await?;
        assert_eq!(client.device_status(auth(wire::DeviceRequest { attempt_id: attempt }, &actor)?).await.err().ok_or("Logged-out session read device state")?.code(), Code::Unauthenticated);
        Ok(())
    }))
}
