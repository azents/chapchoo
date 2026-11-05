//! Synthetic HTTP service under the supported kube::Client, never a live cluster.
use chapchoo_kubernetes::{Config, Error, Kubernetes, Presence};
use chapchoo_protocol::control::{self as wire, ResourceKind};
use http::{Method, Request, Response};
use http_body_util::BodyExt;
use kube::{Client, client::Body};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    convert::Infallible,
    error::Error as _,
    sync::{Arc, Mutex},
};
type TestError = Box<dyn std::error::Error + Send + Sync>;
struct Reply {
    method: Method,
    suffix: &'static str,
    status: u16,
    body: Value,
}
fn reply(method: Method, suffix: &'static str, status: u16, body: Value) -> Reply {
    Reply {
        method,
        suffix,
        status,
        body,
    }
}
type Requests = Arc<Mutex<Vec<(Method, String, Value)>>>;
fn fixture(replies: Vec<Reply>) -> Result<(Kubernetes, Requests), TestError> {
    let replies = Arc::new(Mutex::new(VecDeque::from(replies)));
    let calls = Arc::new(Mutex::new(vec![]));
    let requests = calls.clone();
    let service = tower::service_fn(move |request: Request<Body>| {
        let replies = replies.clone();
        let calls = calls.clone();
        async move {
            let method = request.method().clone();
            let path = request.uri().path().to_owned();
            let uri = request.uri().to_string();
            let bytes = request
                .into_body()
                .collect()
                .await
                .map(|v| v.to_bytes())
                .unwrap_or_default();
            let body = if bytes.is_empty() {
                Value::Null
            } else {
                serde_json::from_slice(&bytes).unwrap_or(Value::Null)
            };
            if let Ok(mut calls) = calls.lock() {
                calls.push((method.clone(), uri, body.clone()));
            }
            let reply = replies.lock().ok().and_then(|mut q| q.pop_front());
            let (status, body) = match reply {
                Some(reply) if reply.method == method && path.ends_with(reply.suffix) => {
                    if reply.status == 201 && reply.body.is_null() {
                        let mut created = body;
                        created["metadata"]["uid"] = json!("created");
                        (201, created)
                    } else {
                        (reply.status, reply.body)
                    }
                }
                _ => (
                    500,
                    json!({"kind":"Status","apiVersion":"v1","status":"Failure","reason":"UnexpectedRequest","code":500}),
                ),
            };
            let mut response = Response::new(Body::from(body.to_string().into_bytes()));
            *response.status_mut() = http::StatusCode::from_u16(status)
                .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);
            Ok::<_, Infallible>(response)
        }
    });
    let adapter = Kubernetes::new(
        Client::new(service, "fixture"),
        Config {
            namespace: "fixture".into(),
            image: "computer:fixture".into(),
            image_pull_policy: "IfNotPresent".into(),
            storage_class: Some("fixture-storage".into()),
            storage_request: "1Gi".into(),
            saved_mount: "/saved".into(),
            service_account: "computer".into(),
            cpu_request: "100m".into(),
            memory_request: "128Mi".into(),
            cpu_limit: "1".into(),
            memory_limit: "1Gi".into(),
        },
    );
    Ok((adapter?, requests))
}
fn context() -> wire::ComputerContext {
    wire::ComputerContext {
        computer_id: "computer".into(),
        request_id: "request".into(),
        execution_id: "execution".into(),
    }
}
fn resource(kind: ResourceKind, uid: Option<&str>) -> wire::ManagedResource {
    wire::ManagedResource {
        kind: kind as i32,
        namespace: "fixture".into(),
        name: match kind {
            ResourceKind::Pod => "pod",
            ResourceKind::Pvc => "pvc",
            _ => "bootstrap",
        }
        .into(),
        uid: uid.map(str::to_owned),
        execution_id: (kind != ResourceKind::Pvc).then(|| "execution".into()),
        allocation: Some(wire::NativeAllocationWitness {
            computer_id: "computer".into(),
            provider_id: "provider".into(),
            execution_id: (kind != ResourceKind::Pvc).then(|| "execution".into()),
        }),
        absence_confirmed: false,
        native: None,
    }
}
fn metadata(name: &str, uid: &str, execution: bool) -> Value {
    let mut annotations =
        json!({"chapchoo.io/provider":"provider","chapchoo.io/computer":"computer"});
    if execution {
        annotations["chapchoo.io/execution"] = json!("execution");
    }
    json!({"name":name,"namespace":"fixture","uid":uid,"resourceVersion":"opaque-before-delete","annotations":annotations})
}
fn missing() -> Value {
    json!({"kind":"Status","apiVersion":"v1","status":"Failure","reason":"NotFound","code":404})
}
fn pvc(uid: &str) -> Value {
    json!({"apiVersion":"v1","kind":"PersistentVolumeClaim","metadata":metadata("pvc",uid,false)})
}
fn pod(uid: &str, phase: &str) -> Value {
    let mut pod = json!({"apiVersion":"v1","kind":"Pod","metadata":metadata("pod",uid,true),"spec":{"containers":[{"name":"computer-connector"}]},"status":{"phase":phase}});
    if phase == "Succeeded" {
        pod["status"]["containerStatuses"] = json!([{"name":"computer-connector","image":"fixture","imageID":"fixture-id","ready":false,"restartCount":0,"state":{"terminated":{"containerID":"containerd://fixture","exitCode":0,"reason":"Completed","finishedAt":"2026-11-02T00:00:00Z"}}}]);
    }
    pod
}
#[tokio::test]
async fn preserved_witness_survives_current_context_and_is_required_before_io()
-> Result<(), TestError> {
    let mut current = context();
    current.computer_id = "canonical-computer".into();
    current.execution_id = "canonical-execution".into();
    let mut original = resource(ResourceKind::Pod, Some("original-uid"));
    original.execution_id = Some(current.execution_id.clone());
    let (api, calls) = fixture(vec![reply(
        Method::GET,
        "/pods/pod",
        200,
        pod("original-uid", "Running"),
    )])?;
    let observed = api.observe(&current, &original).await?;
    assert_eq!(observed.presence, Presence::Present);
    assert_eq!(calls.lock().map_err(|_| "Request lock")?.len(), 1);
    for invalid in 0..4 {
        let (api, calls) = fixture(vec![])?;
        let mut value = original.clone();
        match invalid {
            0 => value.allocation = None,
            1 => value
                .allocation
                .as_mut()
                .ok_or("Witness")?
                .provider_id
                .clear(),
            2 => value.allocation.as_mut().ok_or("Witness")?.computer_id = "../invalid".into(),
            _ => value.allocation.as_mut().ok_or("Witness")?.execution_id = None,
        }
        assert!(matches!(
            api.observe(&current, &value).await,
            Err(Error::Ownership)
        ));
        assert!(calls.lock().map_err(|_| "Request lock")?.is_empty());
    }
    Ok(())
}
#[tokio::test]
async fn original_storage_is_never_replaced_and_initial_allocation_creates_once()
-> Result<(), TestError> {
    for initial in [false, true] {
        let (api, calls) = fixture(vec![reply(
            Method::GET,
            "/persistentvolumeclaims/pvc",
            404,
            missing(),
        )])?;
        assert!(matches!(
            api.ensure_pvc(
                &context(),
                &resource(ResourceKind::Pvc, Some("original")),
                initial
            )
            .await,
            Err(Error::MissingEnvironment)
        ));
        assert_eq!(calls.lock().map_err(|_| "calls")?.len(), 1);
    }
    let (api, calls) = fixture(vec![
        reply(Method::GET, "/persistentvolumeclaims/pvc", 404, missing()),
        reply(
            Method::POST,
            "/persistentvolumeclaims",
            201,
            pvc("allocated"),
        ),
    ])?;
    let created = api
        .ensure_pvc(&context(), &resource(ResourceKind::Pvc, None), true)
        .await?;
    assert_eq!(created.uid.as_deref(), Some("allocated"));
    let calls = calls.lock().map_err(|_| "calls")?;
    assert_eq!(calls.len(), 2);
    assert!(calls[1].2["metadata"].get("ownerReferences").is_none());
    assert_eq!(calls[1].2["spec"]["storageClassName"], "fixture-storage");
    Ok(())
}
#[tokio::test]
async fn foreign_same_name_and_original_uid_mismatch_are_never_adopted() -> Result<(), TestError> {
    let mut foreign = pvc("foreign");
    foreign["metadata"]["annotations"]["chapchoo.io/computer"] = json!("another-computer");
    for (body, expected) in [(foreign, None), (pvc("foreign"), Some("original"))] {
        let (api, calls) = fixture(vec![reply(
            Method::GET,
            "/persistentvolumeclaims/pvc",
            200,
            body,
        )])?;
        assert!(matches!(
            api.ensure_pvc(&context(), &resource(ResourceKind::Pvc, expected), true)
                .await,
            Err(Error::Ownership)
        ));
        assert_eq!(calls.lock().map_err(|_| "calls")?.len(), 1);
    }
    Ok(())
}
#[tokio::test]
async fn fresh_pod_absence_is_authoritative_without_history_and_lookup_failures_are_errors()
-> Result<(), TestError> {
    let object = resource(ResourceKind::Pod, Some("original"));
    for uid in [None, Some("original")] {
        let (api, calls) = fixture(vec![reply(Method::GET, "/pods/pod", 404, missing())])?;
        let observed = api
            .observe(&context(), &resource(ResourceKind::Pod, uid))
            .await?;
        assert_eq!(observed.presence, Presence::Absent);
        assert!(!observed.running);
        assert_eq!(calls.lock().map_err(|_| "calls")?.len(), 1);
    }
    for code in [403, 500] {
        let (api, _) = fixture(vec![reply(
            Method::GET,
            "/pods/pod",
            code,
            json!({"apiVersion":"v1","kind":"Status","status":"Failure","reason":"Unavailable","code":code}),
        )])?;
        assert!(api.observe(&context(), &object).await.is_err());
    }
    let (api, _) = fixture(vec![reply(
        Method::GET,
        "/pods/pod",
        200,
        pod("foreign", "Running"),
    )])?;
    assert!(matches!(
        api.observe(&context(), &object).await,
        Err(Error::Ownership)
    ));
    Ok(())
}
#[tokio::test]
async fn delete_is_one_uid_preconditioned_submission_never_absence_proof() -> Result<(), TestError>
{
    let (api, calls) = fixture(vec![
        reply(Method::GET, "/pods/pod", 200, pod("original", "Running")),
        reply(
            Method::DELETE,
            "/pods/pod",
            200,
            json!({"apiVersion":"v1","kind":"Status","status":"Success"}),
        ),
    ])?;
    api.delete(&context(), &resource(ResourceKind::Pod, Some("original")))
        .await?;
    let calls = calls.lock().map_err(|_| "calls")?;
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1].2["preconditions"]["uid"], "original");
    assert_ne!(calls[1].2["gracePeriodSeconds"], json!(0));
    Ok(())
}
#[tokio::test]
async fn terminal_non_allocation_proof_requires_fresh_absence_and_never_deletes()
-> Result<(), TestError> {
    for kind in [
        ResourceKind::Pod,
        ResourceKind::Pvc,
        ResourceKind::BootstrapSecret,
    ] {
        let suffix = match kind {
            ResourceKind::Pod => "/pods/pod",
            ResourceKind::Pvc => "/persistentvolumeclaims/pvc",
            _ => "/secrets/bootstrap",
        };
        let mut candidate = resource(kind, None);
        candidate.absence_confirmed = true;
        let (api, calls) = fixture(vec![reply(Method::GET, suffix, 404, missing())])?;
        let observation = if kind == ResourceKind::Pod {
            api.retire_pod(&context(), &candidate).await?
        } else {
            api.observe(&context(), &candidate).await?
        };
        assert_eq!(observation.presence, Presence::Absent);
        assert_eq!(calls.lock().map_err(|_| "calls")?.len(), 1);
        let object = match kind {
            ResourceKind::Pod => pod("unexpected", "Running"),
            ResourceKind::Pvc => pvc("unexpected"),
            _ => {
                json!({"apiVersion":"v1","kind":"Secret","metadata":metadata("bootstrap","unexpected",true)})
            }
        };
        let (api, _) = fixture(vec![reply(Method::GET, suffix, 200, object)])?;
        assert!(matches!(
            api.observe(&context(), &candidate).await,
            Err(Error::Ownership)
        ));
    }
    Ok(())
}

#[tokio::test]
async fn asynchronous_cleanup_waits_for_absence_without_resubmitting_delete()
-> Result<(), TestError> {
    for replaced in [false, true] {
        let (api, calls) = fixture(vec![
            reply(
                Method::GET,
                "/persistentvolumeclaims/pvc",
                200,
                pvc("original"),
            ),
            reply(
                Method::DELETE,
                "/persistentvolumeclaims/pvc",
                200,
                json!({"apiVersion":"v1","kind":"Status","status":"Success"}),
            ),
            reply(
                Method::GET,
                "/persistentvolumeclaims/pvc",
                200,
                pvc("original"),
            ),
            reply(
                Method::GET,
                "/persistentvolumeclaims/pvc",
                if replaced { 200 } else { 404 },
                if replaced { pvc("foreign") } else { missing() },
            ),
        ])?;
        let result = api
            .delete(&context(), &resource(ResourceKind::Pvc, Some("original")))
            .await;
        if replaced {
            assert!(matches!(result, Err(Error::Ownership)));
        } else {
            result?;
        }
        let calls = calls.lock().map_err(|_| "calls")?;
        assert_eq!(calls.len(), 4);
        assert_eq!(calls.iter().filter(|c| c.0 == Method::DELETE).count(), 1);
    }
    Ok(())
}

#[tokio::test]
async fn creation_rejection_is_distinct_from_unknown_without_automatic_retry()
-> Result<(), TestError> {
    for (kind, suffix, collection) in [
        (
            ResourceKind::Pvc,
            "/persistentvolumeclaims/pvc",
            "/persistentvolumeclaims",
        ),
        (
            ResourceKind::BootstrapSecret,
            "/secrets/bootstrap",
            "/secrets",
        ),
        (ResourceKind::Pod, "/pods/pod", "/pods"),
    ] {
        for status in [403, 503] {
            let private = "Synthetic private API response";
            let mut replies = Vec::new();
            if kind == ResourceKind::Pod {
                replies.extend([
                reply(Method::GET, "/persistentvolumeclaims/pvc", 200, pvc("pvc-uid")),
                reply(Method::GET, "/secrets/bootstrap", 200,
                    json!({"apiVersion":"v1","kind":"Secret","metadata":metadata("bootstrap","secret-uid",true)})),
            ]);
            }
            replies.extend([
            reply(Method::GET, suffix, 404, missing()),
            reply(
                Method::POST,
                collection,
                status,
                json!({"apiVersion":"v1","kind":"Status","status":"Failure","reason":"Rejected","message":private,"code":status}),
            ),
        ]);
            let (api, calls) = fixture(replies)?;
            let result = match kind {
                ResourceKind::Pvc => {
                    api.ensure_pvc(&context(), &resource(kind, None), true)
                        .await
                }
                ResourceKind::BootstrapSecret => {
                    api.ensure_bootstrap(
                        &context(),
                        &resource(kind, None),
                        "session",
                        "fixture-initial-key",
                        "http://control.invalid",
                    )
                    .await
                }
                ResourceKind::Pod => {
                    api.ensure_pod(
                        &context(),
                        &resource(kind, None),
                        &resource(ResourceKind::Pvc, Some("pvc-uid")),
                        &resource(ResourceKind::BootstrapSecret, Some("secret-uid")),
                    )
                    .await
                }
                _ => return Err("Unsupported fixture kind".into()),
            };
            let error = result.err().ok_or("Creation unexpectedly succeeded")?;
            assert_eq!(error.definitely_rejected(), status == 403);
            let source = error
                .source()
                .and_then(|source| source.downcast_ref::<kube::Error>())
                .ok_or("Missing original SDK cause")?;
            let kube::Error::Api(response) = source else {
                return Err("Missing original API response".into());
            };
            assert_eq!(response.code, status);
            assert_eq!(response.message, private);
            assert!(!error.to_string().contains(private));
            assert_eq!(format!("{error:?}"), "KubernetesError([REDACTED])");
            let calls = calls.lock().map_err(|_| "calls")?;
            assert_eq!(
                calls.iter().filter(|call| call.0 == Method::POST).count(),
                1
            );
            assert_eq!(calls.len(), if kind == ResourceKind::Pod { 4 } else { 2 });
        }
    }
    // GET rejection after an AlreadyExists collision is not a rejected POST:
    // an object may already exist, so it cannot authorize preparation replay.
    let (api, _) = fixture(vec![
        reply(Method::GET, "/persistentvolumeclaims/pvc", 404, missing()),
        reply(
            Method::POST,
            "/persistentvolumeclaims",
            409,
            json!({"apiVersion":"v1","kind":"Status","status":"Failure","reason":"AlreadyExists","code":409}),
        ),
        reply(
            Method::GET,
            "/persistentvolumeclaims/pvc",
            403,
            json!({"apiVersion":"v1","kind":"Status","status":"Failure","reason":"Forbidden","code":403}),
        ),
    ])?;
    assert!(
        !api.ensure_pvc(&context(), &resource(ResourceKind::Pvc, None), true)
            .await
            .err()
            .ok_or("Collision unexpectedly succeeded")?
            .definitely_rejected()
    );
    Ok(())
}

#[tokio::test]
async fn pod_creation_is_last_and_uses_the_verified_independent_pvc_and_bootstrap()
-> Result<(), TestError> {
    let (api, calls) = fixture(vec![
        reply(
            Method::GET,
            "/persistentvolumeclaims/pvc",
            200,
            pvc("pvc-uid"),
        ),
        reply(
            Method::GET,
            "/secrets/bootstrap",
            200,
            json!({"apiVersion":"v1","kind":"Secret","metadata":metadata("bootstrap","secret-uid",true)}),
        ),
        reply(Method::GET, "/pods/pod", 404, missing()),
        reply(Method::POST, "/pods", 201, Value::Null),
    ])?;
    let pod = api
        .ensure_pod(
            &context(),
            &resource(ResourceKind::Pod, None),
            &resource(ResourceKind::Pvc, Some("pvc-uid")),
            &resource(ResourceKind::BootstrapSecret, Some("secret-uid")),
        )
        .await?;
    assert_eq!(pod.uid.as_deref(), Some("created"));
    let valid = {
        let calls = calls.lock().map_err(|_| "calls")?;
        assert_eq!(calls.len(), 4);
        assert_eq!(calls[3].0, Method::POST);
        assert!(calls[3].2["metadata"].get("ownerReferences").is_none());
        assert_eq!(
            calls[3].2["spec"]["volumes"][0]["persistentVolumeClaim"]["claimName"],
            "pvc"
        );
        assert_eq!(calls[3].2["spec"]["automountServiceAccountToken"], false);
        assert_eq!(
            calls[3].2["spec"]["containers"][0]["args"],
            json!(["computer-connector"])
        );
        assert_eq!(
            calls[3].2["spec"]["containers"][0]["env"][0]["valueFrom"]["secretKeyRef"]["name"],
            "bootstrap"
        );
        let mut valid = calls[3].2.clone();
        valid["metadata"]["uid"] = json!("created");
        valid
    };
    for container_override in [false, true] {
        let mut insecure = valid.clone();
        if container_override {
            insecure["spec"]["containers"][0]["securityContext"]["seccompProfile"] =
                json!({"type":"Unconfined"});
        } else {
            insecure["spec"]["securityContext"]["seccompProfile"] = json!({"type":"Unconfined"});
        }
        let (api, calls) = fixture(vec![
            reply(
                Method::GET,
                "/persistentvolumeclaims/pvc",
                200,
                pvc("pvc-uid"),
            ),
            reply(
                Method::GET,
                "/secrets/bootstrap",
                200,
                json!({"apiVersion":"v1","kind":"Secret","metadata":metadata("bootstrap","secret-uid",true)}),
            ),
            reply(Method::GET, "/pods/pod", 200, insecure),
        ])?;
        assert!(matches!(
            api.ensure_pod(
                &context(),
                &resource(ResourceKind::Pod, Some("created")),
                &resource(ResourceKind::Pvc, Some("pvc-uid")),
                &resource(ResourceKind::BootstrapSecret, Some("secret-uid"))
            )
            .await,
            Err(Error::Ownership)
        ));
        assert_eq!(calls.lock().map_err(|_| "calls")?.len(), 3);
    }
    Ok(())
}

#[tokio::test]
async fn normal_retirement_observes_exact_deletion_without_requiring_terminal_status()
-> Result<(), TestError> {
    for phase in ["Succeeded", "Failed"] {
        let (api, calls) = fixture(vec![
            reply(Method::GET, "/pods/pod", 200, pod("original", "Running")),
            reply(
                Method::GET,
                "/pods",
                200,
                json!({"type":"DELETED","object":pod("original",phase)}),
            ),
            reply(
                Method::DELETE,
                "/pods/pod",
                200,
                json!({"apiVersion":"v1","kind":"Status","status":"Success"}),
            ),
        ])?;
        let result = api
            .retire_pod(&context(), &resource(ResourceKind::Pod, Some("original")))
            .await?;
        assert_eq!(result.presence, Presence::Absent);
        let calls = calls.lock().map_err(|_| "calls")?;
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[1].0, Method::GET);
        assert!(calls[1].1.contains("resourceVersion=opaque-before-delete"));
        assert!(calls[1].1.contains("watch=true"));
        assert_eq!(calls[2].0, Method::DELETE);
        assert_eq!(calls[2].2["preconditions"]["uid"], "original");
    }
    Ok(())
}

#[tokio::test]
async fn missing_pod_needs_no_delete_and_compacted_watch_does_not_prove_absence()
-> Result<(), TestError> {
    let (api, calls) = fixture(vec![reply(Method::GET, "/pods/pod", 404, missing())])?;
    assert_eq!(
        api.retire_pod(&context(), &resource(ResourceKind::Pod, Some("original")))
            .await?
            .presence,
        Presence::Absent
    );
    assert_eq!(calls.lock().map_err(|_| "calls")?.len(), 1);
    let (api, calls) = fixture(vec![
        reply(Method::GET, "/pods/pod", 200, pod("original", "Running")),
        reply(
            Method::GET,
            "/pods",
            200,
            json!({"type":"ERROR","object":{"apiVersion":"v1","kind":"Status","reason":"Expired","code":410}}),
        ),
        reply(
            Method::DELETE,
            "/pods/pod",
            200,
            json!({"apiVersion":"v1","kind":"Status","status":"Success"}),
        ),
    ])?;
    assert_eq!(
        api.retire_pod(&context(), &resource(ResourceKind::Pod, Some("original")))
            .await?
            .presence,
        Presence::Unknown
    );
    assert_eq!(
        calls
            .lock()
            .map_err(|_| "calls")?
            .iter()
            .filter(|c| c.0 == Method::DELETE)
            .count(),
        1
    );
    Ok(())
}

#[tokio::test]
async fn fresh_absence_recovers_after_lost_report_without_replaying_delete() -> Result<(), TestError>
{
    for (event, expected) in [
        ("DELETED", Presence::Absent),
        ("MODIFIED", Presence::Unknown),
    ] {
        let (api, calls) = fixture(vec![
            reply(Method::GET, "/pods/pod", 200, pod("original", "Running")),
            reply(
                Method::GET,
                "/pods",
                200,
                json!({"type":event,"object":pod("original","Succeeded")}),
            ),
            reply(
                Method::DELETE,
                "/pods/pod",
                200,
                json!({"apiVersion":"v1","kind":"Status","status":"Success"}),
            ),
            reply(Method::GET, "/pods/pod", 404, missing()),
            reply(Method::GET, "/pods/pod", 404, missing()),
        ])?;
        let object = resource(ResourceKind::Pod, Some("original"));
        let retired = api.retire_pod(&context(), &object).await?;
        assert_eq!(retired.presence, expected);
        assert!(retired.terminal);
        // Do not pass the lost result or committed absence to the new worker.
        let reconnected = api.clone();
        assert_eq!(
            reconnected.observe(&context(), &object).await?.presence,
            Presence::Absent
        );
        let mut successor = context();
        successor.execution_id = "successor".into();
        let mut successor_object = object.clone();
        successor_object.execution_id = Some(successor.execution_id.clone());
        successor_object
            .allocation
            .as_mut()
            .ok_or("Allocation")?
            .execution_id = Some(successor.execution_id.clone());
        assert_eq!(
            reconnected
                .observe(&successor, &successor_object)
                .await?
                .presence,
            Presence::Absent
        );
        assert_eq!(
            calls
                .lock()
                .map_err(|_| "calls")?
                .iter()
                .filter(|call| call.0 == Method::DELETE)
                .count(),
            1
        );
    }
    Ok(())
}

#[tokio::test]
async fn fresh_absence_recovers_after_failed_delete_submission() -> Result<(), TestError> {
    let (api, _) = fixture(vec![
        reply(Method::GET, "/pods/pod", 200, pod("original", "Succeeded")),
        reply(
            Method::GET,
            "/pods",
            200,
            json!({"type":"ERROR","object":{"apiVersion":"v1","kind":"Status","code":410,"reason":"Expired"}}),
        ),
        reply(
            Method::DELETE,
            "/pods/pod",
            500,
            json!({"apiVersion":"v1","kind":"Status","reason":"InternalError","code":500}),
        ),
        reply(Method::GET, "/pods/pod", 404, missing()),
    ])?;
    let object = resource(ResourceKind::Pod, Some("original"));
    assert!(api.retire_pod(&context(), &object).await.is_err());
    assert_eq!(
        api.observe(&context(), &object).await?.presence,
        Presence::Absent
    );
    Ok(())
}

#[tokio::test]
async fn fresh_absence_is_independent_of_prior_watch_content() -> Result<(), TestError> {
    let mut node_lost = pod("original", "Succeeded");
    node_lost["status"]["reason"] = json!("NodeLost");
    for event in [
        pod("original", "Running"),
        pod("original", "Failed"),
        pod("foreign", "Succeeded"),
        node_lost,
    ] {
        let (api, _) = fixture(vec![
            reply(Method::GET, "/pods/pod", 200, pod("original", "Running")),
            reply(
                Method::GET,
                "/pods",
                200,
                json!({"type":"DELETED","object":event.clone()}),
            ),
            reply(
                Method::DELETE,
                "/pods/pod",
                200,
                json!({"apiVersion":"v1","kind":"Status","status":"Success"}),
            ),
            reply(Method::GET, "/pods/pod", 404, missing()),
        ])?;
        let object = resource(ResourceKind::Pod, Some("original"));
        assert_eq!(
            api.retire_pod(&context(), &object).await?.presence,
            if event["metadata"]["uid"] == "original" {
                Presence::Absent
            } else {
                Presence::Unknown
            }
        );
        assert_eq!(
            api.observe(&context(), &object).await?.presence,
            Presence::Absent
        );
    }
    Ok(())
}
