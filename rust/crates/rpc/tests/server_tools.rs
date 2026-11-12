mod support;
use chapchoo_crypto::{BearerSecret, SecretDigest};
use chapchoo_protocol::{
    conversations::CaptureSource,
    server_tools::{self as wire, server_tools_client::ServerToolsClient},
};
use chapchoo_rpc::ServerToolServices;
use chapchoo_service::{
    DomainReason, ServerToolBackend, ServerToolFuture, ServerToolRequest, ServerToolScope,
    ServerToolValidation, ServerTools, ServiceError, ToolCancellation,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tokio::sync::{Notify, Semaphore, mpsc};
use tokio_stream::wrappers::{ReceiverStream, TcpListenerStream};
use tonic::{Request, transport::Server};

type TestError = Box<dyn std::error::Error + Send + Sync>;
struct Backend {
    credential: SecretDigest,
    authorization: Option<Semaphore>,
    gate: Semaphore,
    shutdown: Semaphore,
    effects: AtomicUsize,
    changed: Notify,
}
impl ServerToolBackend for Backend {
    fn authorize<'a>(&'a self, scope: &'a ServerToolScope) -> ServerToolFuture<'a, ()> {
        Box::pin(async move {
            if scope.credential.matches_digest(&self.credential)
                && scope.agent_id == "agent"
                && scope.conversation_id == "conversation"
                && scope.session_id == "session"
                && scope.control_id == "control"
                && scope.connection_id == "connection"
            {
                if let Some(gate) = &self.authorization {
                    gate.acquire()
                        .await
                        .map_err(ServiceError::infrastructure)?
                        .forget();
                }
                Ok(())
            } else {
                Err(ServiceError::domain(DomainReason::PermissionDenied))
            }
        })
    }
    fn catalog(&self) -> Result<Vec<u8>, ServiceError> {
        Ok(br#"{"tools":[{"name":"send_message","inputSchema":{"type":"object"}}]}"#.to_vec())
    }
    fn validate(&self, name: &str, _: &[u8]) -> Result<(), ServerToolValidation> {
        if name == "send_message" {
            Ok(())
        } else {
            Err(ServerToolValidation::UnknownTool)
        }
    }
    fn execute<'a>(
        &'a self,
        request: &'a ServerToolRequest,
        cancel: ToolCancellation,
    ) -> ServerToolFuture<'a, Vec<u8>> {
        Box::pin(async move {
            if request.arguments == b"cancel" {
                tokio::select! {
                    _ = cancel.cancelled() => (),
                    permit = self.shutdown.acquire() => {
                        permit.map_err(ServiceError::infrastructure)?.forget();
                    }
                }
                return Ok(br#"{"content":[{"type":"text","text":"Explicitly cancelled"}],"isError":true}"#.to_vec());
            }
            let permit = self
                .gate
                .acquire()
                .await
                .map_err(ServiceError::infrastructure)?;
            permit.forget();
            self.effects.fetch_add(1, Ordering::AcqRel);
            self.changed.notify_one();
            Ok(br#"{"content":[{"type":"text","text":"Accepted"}],"structuredContent":{"message_id":"fixture","exact":9223372036854775807},"isError":false}"#.to_vec())
        })
    }
}
fn scope() -> wire::CallerScope {
    wire::CallerScope {
        source: Some(CaptureSource {
            session_id: "session".into(),
            control_id: "control".into(),
            connection_id: "connection".into(),
        }),
        agent_id: "agent".into(),
        conversation_id: "conversation".into(),
    }
}
fn auth<T>(message: T, key: &BearerSecret) -> Result<Request<T>, TestError> {
    let mut request = Request::new(message);
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {}", key.expose()).parse()?);
    Ok(request)
}
fn submit(arguments: &[u8]) -> wire::InvocationControl {
    wire::InvocationControl {
        control: Some(wire::invocation_control::Control::Submit(wire::Submit {
            scope: Some(scope()),
            name: "send_message".into(),
            arguments_json: arguments.to_vec(),
        })),
    }
}
#[tokio::test]
async fn generated_component_stream_preserves_scope_result_and_request_local_control()
-> Result<(), TestError> {
    support::scope(async |fixture| {
        let key = BearerSecret::generate()?;
        let wrong = BearerSecret::generate()?;
        let backend = Arc::new(Backend {
            credential: key.digest(),
            authorization: None,
            gate: Semaphore::new(0),
            shutdown: Semaphore::new(0),
            effects: AtomicUsize::new(0),
            changed: Notify::new(),
        });
        let admission = Arc::new(AtomicBool::new(true));
        let adapter = ServerToolServices::new(ServerTools::new(backend.clone()), admission.clone());
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let stop = fixture.shutdown();
        let cleanup_backend = backend.clone();
        let cleanup_adapter = adapter.clone();
        fixture.server(async move {
            let result = Server::builder()
                .add_service(adapter.server())
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
                    let _ = stop.await;
                    admission.store(false, Ordering::Release);
                    cleanup_backend.gate.add_permits(144);
                    cleanup_backend.shutdown.add_permits(144);
                })
                .await;
            let cleanup = cleanup_adapter.finish().await;
            result?;
            cleanup?;
            Ok::<(), TestError>(())
        });
        // No generic ten-second unary helper owns this invocation's lifetime.
        let mut client = ServerToolsClient::connect(format!("http://{address}")).await?;
        assert!(
            client
                .catalog(auth(
                    wire::CatalogRequest {
                        scope: Some(scope())
                    },
                    &wrong
                )?)
                .await
                .is_err()
        );
        assert!(
            client
                .catalog(wire::CatalogRequest {
                    scope: Some(scope())
                })
                .await
                .is_err()
        );
        let catalog = client
            .catalog(auth(
                wire::CatalogRequest {
                    scope: Some(scope()),
                },
                &key,
            )?)
            .await?
            .into_inner();
        assert!(String::from_utf8(catalog.catalog_json)?.contains("send_message"));

        let (sender, receiver) = mpsc::channel(2);
        sender.send(submit(b"publish")).await?;
        let mut stream = client
            .invoke(auth(ReceiverStream::new(receiver), &key)?)
            .await?
            .into_inner();
        assert!(matches!(
            stream.message().await?.and_then(|v| v.observation),
            Some(wire::invocation_observation::Observation::Started(()))
        ));
        backend.gate.add_permits(1);
        let result = stream.message().await?.ok_or("Missing final")?;
        let Some(wire::invocation_observation::Observation::Final(wire::Final {
            outcome: Some(wire::r#final::Outcome::ResultJson(bytes)),
        })) = result.observation
        else {
            return Err("Wrong final".into());
        };
        assert!(String::from_utf8(bytes)?.contains("9223372036854775807"));
        assert!(stream.message().await?.is_none());
        drop(sender);

        let (sender, receiver) = mpsc::channel(2);
        sender.send(submit(b"cancel")).await?;
        let mut stream = client
            .invoke(auth(ReceiverStream::new(receiver), &key)?)
            .await?
            .into_inner();
        assert!(matches!(
            stream.message().await?.and_then(|v| v.observation),
            Some(wire::invocation_observation::Observation::Started(()))
        ));
        sender
            .send(wire::InvocationControl {
                control: Some(wire::invocation_control::Control::Cancel(())),
            })
            .await?;
        assert!(matches!(
            stream.message().await?.and_then(|v| v.observation),
            Some(wire::invocation_observation::Observation::Final(
                wire::Final {
                    outcome: Some(wire::r#final::Outcome::ResultJson(_)),
                }
            ))
        ));
        drop(sender);

        let (sender, receiver) = mpsc::channel(2);
        sender.send(submit(b"uncertain")).await?;
        let mut stream = client
            .invoke(auth(ReceiverStream::new(receiver), &key)?)
            .await?
            .into_inner();
        assert!(matches!(
            stream.message().await?.and_then(|v| v.observation),
            Some(wire::invocation_observation::Observation::Started(()))
        ));
        sender.send(submit(b"second-submit")).await?;
        let error = stream
            .message()
            .await
            .err()
            .ok_or("Accepted second Submit")?;
        assert!(error.message().contains("earlier effect may have occurred"));
        // Breaking the stream never recalls the first invocation or starts the second.
        backend.gate.add_permits(1);
        while backend.effects.load(Ordering::Acquire) != 2 {
            backend.changed.notified().await;
        }
        drop(sender);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn opening_rejections_preserve_invalid_envelope_capacity_and_closing() -> Result<(), TestError>
{
    support::scope(async |fixture| {
        let key = BearerSecret::generate()?;
        let backend = Arc::new(Backend {
            credential: key.digest(),
            authorization: Some(Semaphore::new(0)),
            gate: Semaphore::new(0),
            shutdown: Semaphore::new(0),
            effects: AtomicUsize::new(0),
            changed: Notify::new(),
        });
        let owner = ServerTools::new(backend.clone());
        let admission = Arc::new(AtomicBool::new(true));
        let adapter = ServerToolServices::new(owner.clone(), admission.clone());
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let stop = fixture.shutdown();
        let cleanup_owner = owner.clone();
        let cleanup_admission = admission.clone();
        let cleanup_backend = backend.clone();
        fixture.server(async move {
            let result = Server::builder()
                .add_service(adapter.server())
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
                    let _ = stop.await;
                    cleanup_admission.store(false, Ordering::Release);
                    cleanup_owner.close();
                    if let Some(gate) = &cleanup_backend.authorization {
                        gate.add_permits(144);
                    }
                    cleanup_backend.gate.add_permits(144);
                    cleanup_backend.shutdown.add_permits(144);
                })
                .await;
            let cleanup = adapter.finish().await;
            result?;
            cleanup?;
            Ok::<(), TestError>(())
        });
        let mut client = ServerToolsClient::connect(format!("http://{address}")).await?;
        let mut invalid = submit(b"publish");
        if let Some(wire::invocation_control::Control::Submit(value)) = &mut invalid.control {
            value.name = "x".repeat(129);
        }
        let Err(error) = client
            .invoke(auth(tokio_stream::iter([invalid]), &key)?)
            .await
        else {
            return Err("Invalid envelope unexpectedly admitted".into());
        };
        assert_eq!(
            error
                .metadata()
                .get(wire::NOT_EXECUTED_METADATA)
                .and_then(|v| v.to_str().ok()),
            Some(wire::Rejection::InvalidEnvelope.as_str_name())
        );
        // All preauthorization owners remain physically retained at their gate;
        // no sleeps or scheduler assumptions establish the capacity boundary.
        let mut held = Vec::new();
        for _ in 0..144 {
            held.push(owner.submit(ServerToolRequest {
                scope: ServerToolScope {
                    session_id: "session".into(),
                    control_id: "control".into(),
                    connection_id: "connection".into(),
                    credential: key.digest(),
                    agent_id: "agent".into(),
                    conversation_id: "conversation".into(),
                },
                name: "send_message".into(),
                arguments: b"publish".to_vec(),
            })?);
        }
        let Err(error) = client
            .invoke(auth(tokio_stream::iter([submit(b"publish")]), &key)?)
            .await
        else {
            return Err("Full owner unexpectedly admitted".into());
        };
        assert_eq!(
            error
                .metadata()
                .get(wire::NOT_EXECUTED_METADATA)
                .and_then(|v| v.to_str().ok()),
            Some(wire::Rejection::Capacity.as_str_name())
        );
        admission.store(false, Ordering::Release);
        let Err(error) = client
            .invoke(auth(tokio_stream::iter([submit(b"publish")]), &key)?)
            .await
        else {
            return Err("Closing service unexpectedly admitted".into());
        };
        assert_eq!(
            error
                .metadata()
                .get(wire::NOT_EXECUTED_METADATA)
                .and_then(|v| v.to_str().ok()),
            Some(wire::Rejection::Closing.as_str_name())
        );
        assert_eq!(backend.effects.load(Ordering::Acquire), 0);
        drop(held);
        Ok(())
    })
    .await
}
