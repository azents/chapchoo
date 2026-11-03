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
        "provider".into(),
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
async fn pod_absence_requires_immutable_terminal_evidence_not_a_bare_404() -> Result<(), TestError>
{
    let object = resource(ResourceKind::Pod, Some("original"));
    let (failed, _) = fixture(vec![reply(
        Method::GET,
        "/pods/pod",
        200,
        pod("original", "Failed"),
    )])?;
    assert!(
        failed
            .observe(&context(), &object, None)
            .await?
            .terminal
            .is_none()
    );
    let mut lost = pod("original", "Succeeded");
    lost["status"]["containerStatuses"][0]["state"]["terminated"]["reason"] =
        json!("ContainerStatusUnknown");
    let (lost, _) = fixture(vec![reply(Method::GET, "/pods/pod", 200, lost)])?;
    assert!(
        lost.observe(&context(), &object, None)
            .await?
            .terminal
            .is_none()
    );
    let (api, _) = fixture(vec![reply(Method::GET, "/pods/pod", 404, missing())])?;
    assert_eq!(
        api.observe(&context(), &object, None).await?.presence,
        Presence::Unknown
    );
    let (api, _) = fixture(vec![
        reply(Method::GET, "/pods/pod", 200, pod("original", "Succeeded")),
        reply(Method::GET, "/pods/pod", 404, missing()),
        reply(Method::GET, "/pods/pod", 404, missing()),
    ])?;
    let observed = api.observe(&context(), &object, None).await?;
    let witness = observed.terminal.ok_or("Terminal witness")?;
    assert_eq!(
        api.observe(&context(), &object, Some(&witness))
            .await?
            .presence,
        Presence::Absent
    );
    let foreign = resource(ResourceKind::Pod, Some("different"));
    assert_eq!(
        api.observe(&context(), &foreign, Some(&witness))
            .await?
            .presence,
        Presence::Unknown
    );
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
            api.observe(&context(), &candidate, None).await?
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
            api.observe(&context(), &candidate, None).await,
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
    for status in [403, 503] {
        let (api, calls) = fixture(vec![
            reply(Method::GET, "/persistentvolumeclaims/pvc", 404, missing()),
            reply(
                Method::POST,
                "/persistentvolumeclaims",
                status,
                json!({"apiVersion":"v1","kind":"Status","status":"Failure","reason":"Rejected","code":status}),
            ),
        ])?;
        let error = api
            .ensure_pvc(&context(), &resource(ResourceKind::Pvc, None), true)
            .await
            .err()
            .ok_or("Creation unexpectedly succeeded")?;
        assert_eq!(error.definitely_rejected(), status == 403);
        assert_eq!(calls.lock().map_err(|_| "calls")?.len(), 2);
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
async fn normal_retirement_starts_ordered_watch_before_uid_delete_and_requires_terminal_event()
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
        assert_eq!(
            result.presence,
            if phase == "Succeeded" {
                Presence::Absent
            } else {
                Presence::Unknown
            }
        );
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
async fn missing_pod_and_compacted_watch_never_authorize_retirement_or_delete_replay()
-> Result<(), TestError> {
    let (api, calls) = fixture(vec![reply(Method::GET, "/pods/pod", 404, missing())])?;
    assert_eq!(
        api.retire_pod(&context(), &resource(ResourceKind::Pod, Some("original")))
            .await?
            .presence,
        Presence::Unknown
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
