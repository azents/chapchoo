use super::*;
#[path = "device/refresh.rs"]
mod refresh;
use chapchoo_crypto::SecretDigest;
use chapchoo_persistence::{
    CheckDeviceAdmission, DeviceAuthenticationState as State, DeviceAuthenticationStatus,
    DeviceAuthenticationStep, DeviceAuthenticationTokens, DeviceAuthorizationDraft,
    DeviceStepFailure, DeviceStepSettlement, ExchangeDeviceAdmission, ReauthenticateTarget,
    StartDeviceAdmission, StartDeviceAuthentication,
};
use diesel::sql_types::BigInt;

fn tokens(account: Option<&str>) -> Result<DeviceAuthenticationTokens, TestError> {
    Ok(DeviceAuthenticationTokens {
        secret: key("synthetic-oauth-token-bundle")?,
        account_id: account.map(str::to_owned),
        email: Some("synthetic@example.invalid".to_owned()),
        plan_type: Some("synthetic-plan".to_owned()),
        expires_at: Some(NOW + 3_600_000),
    })
}
fn issued(now: i64) -> Result<DeviceAuthorizationDraft, TestError> {
    Ok(DeviceAuthorizationDraft {
        device_auth_id: key("synthetic-device-id")?,
        user_code: "SYNTH-CODE".to_owned(),
        interval_ms: 1000,
        expires_at: now + 60000,
    })
}
fn registered(status: DeviceAuthenticationStatus) -> Result<String, TestError> {
    match status.state {
        State::Registered { authentication_id } => Ok(authentication_id),
        other => Err(format!("Expected registered result, got {other:?}").into()),
    }
}
async fn start(
    store: &Store,
    actor: &SecretDigest,
    scope: &AuthenticationScope,
    request: &str,
    target: Option<(&str, &str)>,
) -> Result<DeviceAuthenticationStep, TestError> {
    let draft = StartDeviceAuthentication {
        scope: scope.clone(),
        request_id: request.to_owned(),
        name: String::new(),
        target: target.map(|(id, expected)| ReauthenticateTarget {
            authentication_id: id.to_owned(),
            expected_request_id: expected.to_owned(),
        }),
    };
    match store
        .start_external_device_authentication(actor, &draft, NOW)
        .await?
    {
        StartDeviceAdmission::Issue(step) => Ok(step),
        _ => Err("Expected a new issuance admission".into()),
    }
}
async fn issue_to_exchange(
    store: &Store,
    actor: &SecretDigest,
    step: DeviceAuthenticationStep,
) -> Result<DeviceAuthenticationStep, TestError> {
    let attempt = step.attempt_id();
    assert!(matches!(
        store
            .accept_external_device_authorization(actor, &step, &issued(NOW)?, NOW)
            .await?
            .state,
        State::Pending { .. }
    ));
    let CheckDeviceAdmission::Check(check) = store
        .check_external_device_authentication(actor, &attempt, NOW + 1000)
        .await?
    else {
        return Err("Missing check admission".into());
    };
    let ExchangeDeviceAdmission::Exchange(step) = store
        .exchange_external_device_authentication(actor, &check.step, NOW + 1000)
        .await?
    else {
        return Err("Missing exchange admission".into());
    };
    Ok(step)
}
async fn exchange(
    store: &Store,
    actor: &SecretDigest,
    scope: &AuthenticationScope,
    request: &str,
    target: Option<(&str, &str)>,
) -> Result<DeviceAuthenticationStep, TestError> {
    issue_to_exchange(
        store,
        actor,
        start(store, actor, scope, request, target).await?,
    )
    .await
}

async fn exercise_device(store: &Store) -> Result<(String, String, String), TestError> {
    let session = seed(store).await?;
    let owner = session[0].digest();
    let manager = session[1].digest();
    let stranger = session[3].digest();
    refresh::exercise(store, &owner, &stranger).await?;
    let personal = AuthenticationScope::Personal;
    let first = start(store, &owner, &personal, "device-success", None).await?;
    let attempt = first.attempt_id();
    let replay = StartDeviceAuthentication {
        scope: personal.clone(),
        request_id: "device-success".to_owned(),
        name: String::new(),
        target: None,
    };
    assert!(matches!(
        store
            .start_external_device_authentication(&owner, &replay, NOW)
            .await?,
        StartDeviceAdmission::Current(_)
    ));
    assert!(matches!(
        store
            .external_device_authentication_status(&stranger, &attempt, NOW)
            .await,
        Err(StoreError::NotFound)
    ));
    let authorization = issued(NOW)?;
    let status = store
        .accept_external_device_authorization(&owner, &first, &authorization, NOW)
        .await?;
    assert_eq!(
        format!("{status:?}"),
        "DeviceAuthenticationStatus([REDACTED])"
    );
    assert!(
        matches!(&status.state,State::Pending { user_code,next_check_at } if user_code=="SYNTH-CODE" && *next_check_at==NOW+1000)
    );
    assert!(matches!(
        store
            .check_external_device_authentication(&owner, &attempt, NOW + 999)
            .await?,
        CheckDeviceAdmission::Current(_)
    ));
    // The DB predicate is the single poll winner on either engine, not a local mutex.
    let (a, b) = tokio::join!(
        store.check_external_device_authentication(&owner, &attempt, NOW + 1000),
        store.check_external_device_authentication(&owner, &attempt, NOW + 1000)
    );
    let check = match (a?, b?) {
        (CheckDeviceAdmission::Check(check), CheckDeviceAdmission::Current(status))
        | (CheckDeviceAdmission::Current(status), CheckDeviceAdmission::Check(check)) => {
            assert!(matches!(status.state, State::Checking));
            check
        }
        _ => return Err("More than one check was admitted".into()),
    };
    assert_eq!(check.device_auth_id.expose(), b"synthetic-device-id");
    assert_eq!(check.user_code.as_str(), "SYNTH-CODE");
    store
        .accept_external_device_pending(&owner, &check.step, 5000, NOW + 1000)
        .await?;
    // A delayed reply from a previous poll cannot admit an exchange after it settled.
    assert!(matches!(
        store
            .exchange_external_device_authentication(&owner, &check.step, NOW + 1001)
            .await?,
        ExchangeDeviceAdmission::Current(_)
    ));
    assert!(matches!(
        store
            .check_external_device_authentication(&owner, &attempt, NOW + 5999)
            .await?,
        CheckDeviceAdmission::Current(_)
    ));
    let CheckDeviceAdmission::Check(next) = store
        .check_external_device_authentication(&owner, &attempt, NOW + 6000)
        .await?
    else {
        return Err("Missing next check".into());
    };
    // An old step's settlement cannot stop a newer admitted step.
    assert!(matches!(
        store
            .fail_external_device_step(&check.step, DeviceStepFailure::Uncertain, NOW + 6000)
            .await?,
        DeviceStepSettlement::AlreadySettled
    ));
    let ExchangeDeviceAdmission::Exchange(step) = store
        .exchange_external_device_authentication(&owner, &next.step, NOW + 6000)
        .await?
    else {
        return Err("Missing exchange".into());
    };
    assert!(matches!(
        store
            .exchange_external_device_authentication(&owner, &next.step, NOW + 6000)
            .await?,
        ExchangeDeviceAdmission::Current(_)
    ));
    // An in-flight exchange is never returned as another dispatchable check.
    assert!(matches!(
        store
            .check_external_device_authentication(&owner, &attempt, NOW + 6001)
            .await?,
        CheckDeviceAdmission::Current(_)
    ));
    let id = registered(
        store
            .complete_external_device_authentication(
                &owner,
                &step,
                &tokens(Some("account-a"))?,
                NOW + 6001,
            )
            .await?,
    )?;
    assert_eq!(
        registered(
            store
                .complete_external_device_authentication(
                    &owner,
                    &step,
                    &tokens(Some("account-a"))?,
                    NOW + 6002
                )
                .await?
        )?,
        id
    );
    assert_eq!(
        registered(
            store
                .cancel_external_device_authentication(
                    &owner,
                    &attempt,
                    "cancel-after-commit",
                    NOW + 6002
                )
                .await?
        )?,
        id
    );
    assert_eq!(
        store
            .external_authentications(&owner, &personal, None, 100, NOW)
            .await?
            .len(),
        1
    );
    assert!(matches!(
        store
            .start_external_device_authentication(&owner, &replay, NOW + 6002)
            .await?,
        StartDeviceAdmission::Current(DeviceAuthenticationStatus {
            state: State::Registered { .. },
            ..
        })
    ));

    // Same-account replacement preserves ID and alias; different/absent identity does not replace it.
    for (request, account) in [
        ("wrong-account", Some("account-b")),
        ("missing-account", None),
    ] {
        let reauth = exchange(
            store,
            &owner,
            &personal,
            request,
            Some((&id, "device-success")),
        )
        .await?;
        assert!(matches!(
            store
                .complete_external_device_authentication(
                    &owner,
                    &reauth,
                    &tokens(account)?,
                    NOW + 1001
                )
                .await?
                .state,
            State::Failed
        ));
        assert_eq!(
            store
                .external_authentication(&owner, &personal, &id, NOW)
                .await?
                .current_request_id,
            "device-success"
        );
    }
    let old = exchange(
        store,
        &owner,
        &personal,
        "superseded-reauth",
        Some((&id, "device-success")),
    )
    .await?;
    let newer = start(
        store,
        &owner,
        &personal,
        "reauth-success",
        Some((&id, "device-success")),
    )
    .await?;
    assert!(matches!(
        store
            .complete_external_device_authentication(
                &owner,
                &old,
                &tokens(Some("account-a"))?,
                NOW + 1001
            )
            .await?
            .state,
        State::Superseded
    ));
    let newer = issue_to_exchange(store, &owner, newer).await?;
    let replacement = DeviceAuthenticationTokens {
        secret: key("replacement-oauth-bundle")?,
        ..tokens(Some("account-a"))?
    };
    assert_eq!(
        registered(
            store
                .complete_external_device_authentication(&owner, &newer, &replacement, NOW + 1001)
                .await?
        )?,
        id
    );
    let detail = store
        .external_authentication(&owner, &personal, &id, NOW)
        .await?;
    assert_eq!(detail.current_request_id, "reauth-success");
    assert_eq!(detail.name, "");
    assert_eq!(detail.account_id.as_deref(), Some("account-a"));

    let uncertain = exchange(store, &owner, &personal, "uncertain-exchange", None).await?;
    store
        .fail_external_device_step(&uncertain, DeviceStepFailure::Uncertain, NOW + 1001)
        .await?;
    assert!(matches!(
        store
            .check_external_device_authentication(&owner, &uncertain.attempt_id(), NOW + 1_000_000)
            .await?,
        CheckDeviceAdmission::Current(DeviceAuthenticationStatus {
            state: State::Uncertain,
            ..
        })
    ));
    assert!(matches!(
        store
            .complete_external_device_authentication(
                &owner,
                &uncertain,
                &tokens(Some("account-a"))?,
                NOW + 1002
            )
            .await?
            .state,
        State::Uncertain
    ));
    assert_eq!(
        store
            .external_authentications(&owner, &personal, None, 100, NOW)
            .await?
            .len(),
        1
    );

    // Cancellation may win at each external boundary; later results cannot publish.
    let cancelled = start(store, &owner, &personal, "cancelled-issue", None).await?;
    store
        .cancel_external_device_authentication(&owner, &cancelled.attempt_id(), "cancel-issue", NOW)
        .await?;
    assert!(matches!(
        store
            .accept_external_device_authorization(&owner, &cancelled, &authorization, NOW + 1)
            .await?
            .state,
        State::Cancelled
    ));
    let cancelled = start(store, &owner, &personal, "cancelled-check", None).await?;
    store
        .accept_external_device_authorization(&owner, &cancelled, &authorization, NOW)
        .await?;
    let CheckDeviceAdmission::Check(check) = store
        .check_external_device_authentication(&owner, &cancelled.attempt_id(), NOW + 1000)
        .await?
    else {
        return Err("Missing cancellable check".into());
    };
    store
        .cancel_external_device_authentication(
            &owner,
            &cancelled.attempt_id(),
            "cancel-check",
            NOW + 1000,
        )
        .await?;
    assert!(matches!(
        store
            .exchange_external_device_authentication(&owner, &check.step, NOW + 1000)
            .await?,
        ExchangeDeviceAdmission::Current(DeviceAuthenticationStatus {
            state: State::Cancelled,
            ..
        })
    ));
    let cancelled = exchange(store, &owner, &personal, "cancelled-exchange", None).await?;
    store
        .cancel_external_device_authentication(
            &owner,
            &cancelled.attempt_id(),
            "cancel-exchange",
            NOW + 1000,
        )
        .await?;
    assert!(matches!(
        store
            .complete_external_device_authentication(
                &owner,
                &cancelled,
                &tokens(Some("account-a"))?,
                NOW + 1001
            )
            .await?
            .state,
        State::Cancelled
    ));
    assert!(matches!(
        store
            .cancel_external_device_authentication(
                &owner,
                &cancelled.attempt_id(),
                "cancel-exchange",
                NOW + 1001
            )
            .await?
            .state,
        State::Cancelled
    ));
    assert!(matches!(
        store
            .cancel_external_device_authentication(&owner, &attempt, "cancel-exchange", NOW + 1001)
            .await,
        Err(StoreError::Conflict)
    ));
    let racing = exchange(store, &owner, &personal, "cancel-race", None).await?;
    let racing_id = racing.attempt_id();
    let token_set = tokens(Some("account-a"))?;
    let (cancellation, completion) = tokio::join!(
        store.cancel_external_device_authentication(
            &owner,
            &racing_id,
            "cancel-race-request",
            NOW + 1001
        ),
        store.complete_external_device_authentication(&owner, &racing, &token_set, NOW + 1001),
    );
    match (cancellation?.state, completion?.state) {
        (State::Cancelled, State::Cancelled) => {}
        (
            State::Registered {
                authentication_id: first,
            },
            State::Registered {
                authentication_id: second,
            },
        ) if first == second => {
            store
                .delete_external_authentication(
                    &owner,
                    &DeleteAuthentication {
                        scope: personal.clone(),
                        request_id: "remove-race-result".to_owned(),
                        authentication_id: first,
                        expected_request_id: "cancel-race".to_owned(),
                    },
                    NOW + 1002,
                )
                .await?;
        }
        _ => return Err("Cancellation and acceptance did not share one committed winner".into()),
    }
    let expired_exchange = exchange(store, &owner, &personal, "expired-exchange", None).await?;
    assert!(matches!(
        store
            .check_external_device_authentication(
                &owner,
                &expired_exchange.attempt_id(),
                NOW + 900001
            )
            .await?,
        CheckDeviceAdmission::Current(DeviceAuthenticationStatus {
            state: State::Expired,
            ..
        })
    ));
    assert!(matches!(
        store
            .complete_external_device_authentication(
                &owner,
                &expired_exchange,
                &tokens(Some("account-a"))?,
                NOW + 900001
            )
            .await?
            .state,
        State::Expired
    ));
    let expired = start(store, &owner, &personal, "expired-device", None).await?;
    let mut short = issued(NOW)?;
    short.expires_at = NOW + 2000;
    store
        .accept_external_device_authorization(&owner, &expired, &short, NOW)
        .await?;
    assert!(matches!(
        store
            .check_external_device_authentication(&owner, &expired.attempt_id(), NOW + 2000)
            .await?,
        CheckDeviceAdmission::Current(DeviceAuthenticationStatus {
            state: State::Expired,
            ..
        })
    ));
    let expired = start(store, &owner, &personal, "expired-issuance", None).await?;
    assert!(matches!(
        store
            .accept_external_device_authorization(&owner, &expired, &short, NOW + 2000)
            .await?
            .state,
        State::Expired
    ));

    let bounded = exchange(store, &owner, &personal, "bounded-expiry", None).await?;
    let before = store
        .external_authentications(&owner, &personal, None, 100, NOW)
        .await?
        .len();
    let mut bounded_tokens = tokens(Some("bounded-expiry-account"))?;
    bounded_tokens.expires_at = Some(chapchoo_persistence::MAX_AUTHENTICATION_EXPIRY_MILLIS + 1);
    assert!(matches!(
        store
            .complete_external_device_authentication(&owner, &bounded, &bounded_tokens, NOW + 1001)
            .await,
        Err(StoreError::InvalidInput)
    ));
    assert_eq!(
        store
            .external_authentications(&owner, &personal, None, 100, NOW)
            .await?
            .len(),
        before
    );
    bounded_tokens.expires_at = Some(chapchoo_persistence::MAX_AUTHENTICATION_EXPIRY_MILLIS);
    let bounded_id = registered(
        store
            .complete_external_device_authentication(&owner, &bounded, &bounded_tokens, NOW + 1001)
            .await?,
    )?;
    assert_eq!(
        store
            .external_authentication(&owner, &personal, &bounded_id, NOW + 1001)
            .await?
            .expires_at,
        Some(chapchoo_persistence::MAX_AUTHENTICATION_EXPIRY_MILLIS)
    );
    assert!(matches!(
        store
            .delete_external_authentication(
                &owner,
                &DeleteAuthentication {
                    scope: personal.clone(),
                    request_id: "delete-bounded-expiry".into(),
                    authentication_id: bounded_id,
                    expected_request_id: "bounded-expiry".into(),
                },
                NOW + 1001
            )
            .await?,
        DeleteAuthenticationOutcome::Accepted(_)
    ));

    // Observation itself terminalizes abandoned temporary material. No later
    // poll/cancel/completion is allowed to incidentally perform this cleanup.
    for phase in ["starting", "pending", "exchanging"] {
        let request = format!("observed-expiry-{phase}");
        let step = start(store, &owner, &personal, &request, None).await?;
        let attempt = step.attempt_id();
        match phase {
            "pending" => {
                store
                    .accept_external_device_authorization(&owner, &step, &issued(NOW)?, NOW)
                    .await?;
            }
            "exchanging" => {
                issue_to_exchange(store, &owner, step).await?;
            }
            _ => {}
        }
        assert!(matches!(
            store
                .external_device_authentication_status(&owner, &attempt, NOW + 900_001)
                .await?
                .state,
            State::Expired
        ));
        assert!(
            store
                .external_authentication_request_result(&owner, &request, NOW + 900_001)
                .await?
                .is_some()
        );
    }
    // Exercise receipt replay separately, so it cannot mask a missing status
    // cleanup for the three abandoned attempts above.
    let replay = start(store, &owner, &personal, "observed-expiry-replay", None).await?;
    store
        .accept_external_device_authorization(&owner, &replay, &issued(NOW)?, NOW)
        .await?;
    assert!(matches!(
        store
            .start_external_device_authentication(
                &owner,
                &StartDeviceAuthentication {
                    scope: personal.clone(),
                    request_id: "observed-expiry-replay".into(),
                    name: String::new(),
                    target: None,
                },
                NOW + 900_001
            )
            .await?,
        StartDeviceAdmission::Current(DeviceAuthenticationStatus {
            state: State::Expired,
            ..
        })
    ));

    let team = store
        .create_team(
            &owner,
            &CreateTeam {
                name: "Device Team".to_owned(),
                handle: "device-team".to_owned(),
            },
            NOW,
        )
        .await?;
    let invitation = store
        .invite_member(
            &owner,
            &InviteMember {
                team_id: team.id.clone(),
                recipient: EmailAddress::parse("auth1@example.invalid")?,
                signup: None,
            },
            NOW,
        )
        .await?;
    store
        .process_invitation(&manager, &invitation.invitation.id, true, NOW)
        .await?;
    let manager_user = store.current_identity(&manager, NOW).await?.id;
    store
        .set_member_role(
            &owner,
            &team.id,
            &manager_user,
            MembershipRole::Manager,
            NOW,
        )
        .await?;
    let shared = AuthenticationScope::Team {
        team_id: team.id.clone(),
    };
    let revoked = exchange(store, &manager, &shared, "role-withdrawal", None).await?;
    store
        .set_member_role(&owner, &team.id, &manager_user, MembershipRole::Member, NOW)
        .await?;
    assert!(matches!(
        store
            .complete_external_device_authentication(
                &manager,
                &revoked,
                &tokens(Some("account-a"))?,
                NOW + 1001
            )
            .await,
        Err(StoreError::Forbidden)
    ));
    // A sealed step can only settle/clear itself even after its session/role is withdrawn.
    assert!(matches!(
        store
            .fail_external_device_step(&revoked, DeviceStepFailure::Failed, NOW + 1001)
            .await?,
        DeviceStepSettlement::Recorded
    ));
    assert!(
        store
            .external_authentications(&owner, &shared, None, 100, NOW)
            .await?
            .is_empty()
    );
    store
        .set_member_role(
            &owner,
            &team.id,
            &manager_user,
            MembershipRole::Manager,
            NOW,
        )
        .await?;
    let departing = exchange(store, &manager, &shared, "departing-device", None).await?;
    store
        .set_member_role(&owner, &team.id, &manager_user, MembershipRole::Member, NOW)
        .await?;
    store
        .remove_team_member(&owner, &team.id, &manager_user, NOW)
        .await?;
    assert!(matches!(
        store
            .complete_external_device_authentication(
                &manager,
                &departing,
                &tokens(Some("account-a"))?,
                NOW + 1001
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .fail_external_device_step(&departing, DeviceStepFailure::Uncertain, NOW + 1001)
            .await?,
        DeviceStepSettlement::AlreadySettled
    ));
    let invitation = store
        .invite_member(
            &owner,
            &InviteMember {
                team_id: team.id.clone(),
                recipient: EmailAddress::parse("auth1@example.invalid")?,
                signup: None,
            },
            NOW,
        )
        .await?;
    store
        .process_invitation(&manager, &invitation.invitation.id, true, NOW)
        .await?;
    store
        .set_member_role(
            &owner,
            &team.id,
            &manager_user,
            MembershipRole::Manager,
            NOW,
        )
        .await?;
    let deleting = exchange(store, &manager, &shared, "team-deleted", None).await?;
    store.delete_team(&owner, &team.id, NOW).await?;
    assert!(matches!(
        store
            .complete_external_device_authentication(
                &manager,
                &deleting,
                &tokens(Some("account-a"))?,
                NOW + 1001
            )
            .await,
        Err(StoreError::NotFound)
    ));
    assert!(matches!(
        store
            .fail_external_device_step(&deleting, DeviceStepFailure::Uncertain, NOW + 1001)
            .await?,
        DeviceStepSettlement::Missing
    ));

    // Deleting a target detaches old attempt references, but cannot turn reauth into new registration.
    let victim = exchange(store, &owner, &personal, "victim-create", None).await?;
    let victim_id = registered(
        store
            .complete_external_device_authentication(
                &owner,
                &victim,
                &tokens(Some("account-a"))?,
                NOW + 1001,
            )
            .await?,
    )?;
    let deleted = exchange(
        store,
        &owner,
        &personal,
        "target-deleted",
        Some((&victim_id, "victim-create")),
    )
    .await?;
    store
        .delete_external_authentication(
            &owner,
            &DeleteAuthentication {
                scope: personal.clone(),
                request_id: "delete-target".to_owned(),
                authentication_id: victim_id,
                expected_request_id: "victim-create".to_owned(),
            },
            NOW,
        )
        .await?;
    assert!(matches!(
        store
            .complete_external_device_authentication(
                &owner,
                &deleted,
                &tokens(Some("account-a"))?,
                NOW + 1001
            )
            .await?
            .state,
        State::Superseded
    ));
    assert_eq!(
        store
            .external_authentications(&owner, &personal, None, 100, NOW)
            .await?
            .len(),
        1
    );
    let revoked_session = exchange(store, &stranger, &personal, "revoked-session", None).await?;
    store.logout_session(&stranger, NOW).await?;
    assert!(matches!(
        store
            .complete_external_device_authentication(
                &stranger,
                &revoked_session,
                &tokens(Some("account-a"))?,
                NOW + 1001
            )
            .await,
        Err(StoreError::Unauthenticated)
    ));
    assert!(matches!(
        store
            .fail_external_device_step(&revoked_session, DeviceStepFailure::Failed, NOW + 1001)
            .await?,
        DeviceStepSettlement::Recorded
    ));
    let final_step = exchange(store, &owner, &personal, "final-device", None).await?;
    let final_id = registered(
        store
            .complete_external_device_authentication(
                &owner,
                &final_step,
                &tokens(Some("account-a"))?,
                NOW + 1001,
            )
            .await?,
    )?;
    Ok((store.current_identity(&owner, NOW).await?.id, final_id, id))
}

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type=BigInt)]
    n: i64,
}
const TERMINAL_SECRETS: &str = "SELECT count(*) AS n FROM external_auth_attempts WHERE (phase NOT IN ('STARTING','PENDING','CHECKING','EXCHANGING') AND (pending_nonce IS NOT NULL OR pending_ciphertext IS NOT NULL OR step_id IS NOT NULL)) OR (request_id IN ('observed-expiry-starting','observed-expiry-pending','observed-expiry-exchanging','observed-expiry-replay') AND phase<>'EXPIRED')";
fn verify_oauth(
    cipher: &CredentialCipher,
    row: StoredSecret,
    owner: &str,
    id: &str,
    expected: &[u8],
) -> Result<(), TestError> {
    let envelope = ExternalSecretEnvelope::from_parts(
        row.nonce.try_into().map_err(|_| "Invalid nonce")?,
        row.ciphertext,
    )?;
    let context = ExternalSecretContext::new(
        ExternalSecretOwner::User(owner),
        id,
        ExternalSecretKind::ChatGpt,
    )?;
    assert_eq!(
        cipher.recover_external(&context, &envelope)?.expose(),
        expected
    );
    Ok(())
}
#[test]
fn sqlite_device_steps_fence_concurrency_cancellation_reauth_and_uncertainty()
-> Result<(), TestError> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("device.sqlite");
    let cipher = cipher()?;
    prepare_sqlite(&path, &cipher)?;
    let (owner, id, reauthenticated) = tokio::runtime::Runtime::new()?.block_on(async {
        let store = Store::connect_sqlite(path.clone(), cipher.clone()).await?;
        exercise_device(&store).await
    })?;
    let mut connection = diesel::SqliteConnection::establish(path.to_str().ok_or("Invalid path")?)?;
    assert_eq!(
        diesel::sql_query(TERMINAL_SECRETS)
            .get_result::<Count>(&mut connection)?
            .n,
        0
    );
    let row=diesel::sql_query("SELECT nonce,ciphertext FROM external_authentications WHERE current_request_id='final-device'").get_result::<StoredSecret>(&mut connection)?;
    verify_oauth(&cipher, row, &owner, &id, b"synthetic-oauth-token-bundle")?;
    let row=diesel::sql_query("SELECT nonce,ciphertext FROM external_authentications WHERE current_request_id='reauth-success'").get_result::<StoredSecret>(&mut connection)?;
    verify_oauth(
        &cipher,
        row,
        &owner,
        &reauthenticated,
        b"replacement-oauth-bundle",
    )
}
#[test]
fn postgres_device_steps_fence_concurrency_cancellation_reauth_and_uncertainty()
-> Result<(), TestError> {
    let _fixture = POSTGRES_FIXTURE_LOCK
        .lock()
        .map_err(|_| "PostgreSQL fixture lock poisoned")?;
    let base = std::env::var("CHAPCHOO_TEST_POSTGRES_URL")?;
    let mut control = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&base)?;
    let suffix = BearerSecret::generate()?.digest().as_bytes()[..8]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let schema = format!("device_steps_{suffix}");
    control.batch_execute(&format!("CREATE SCHEMA {schema}"))?;
    let separator = if base.contains('?') { '&' } else { '?' };
    let url = format!("{base}{separator}options=-csearch_path%3D{schema}");
    let result = (|| -> Result<(), TestError> {
        let cipher = cipher()?;
        prepare_postgres(&url, &cipher)?;
        let (owner, id, reauthenticated) = tokio::runtime::Runtime::new()?.block_on(async {
            let store = Store::connect_postgres(url.clone(), cipher.clone()).await?;
            exercise_device(&store).await
        })?;
        let mut connection = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        assert_eq!(
            diesel::sql_query(TERMINAL_SECRETS)
                .get_result::<Count>(&mut connection)?
                .n,
            0
        );
        let row=diesel::sql_query("SELECT nonce,ciphertext FROM external_authentications WHERE current_request_id='final-device'").get_result::<StoredSecret>(&mut connection)?;
        verify_oauth(&cipher, row, &owner, &id, b"synthetic-oauth-token-bundle")?;
        let row=diesel::sql_query("SELECT nonce,ciphertext FROM external_authentications WHERE current_request_id='reauth-success'").get_result::<StoredSecret>(&mut connection)?;
        verify_oauth(
            &cipher,
            row,
            &owner,
            &reauthenticated,
            b"replacement-oauth-bundle",
        )
    })();
    let cleanup = control.batch_execute(&format!("DROP SCHEMA {schema} CASCADE"));
    support::combine(result, cleanup.map_err(Into::into)).map(|_| ())
}
