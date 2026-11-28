//! Production Connector against a deterministic component RPC peer.
use super::*;
use chapchoo_crypto::SecretDigest;
use chapchoo_domain::{NativeConversationId, SnapshotId};
use chapchoo_harness::NativeTextTarget;
use chapchoo_protocol::server_tools::{self as wire, server_tools_server};
use std::time::Duration;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

struct Control {
    credential: SecretDigest,
    entered: mpsc::Sender<wire::ExternalInvocation>,
    release: Semaphore,
    preparing: Semaphore,
    prepare_release: Semaphore,
}
impl Control {
    fn authorized<T>(&self, request: &Request<T>) -> bool {
        request
            .metadata()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .and_then(|value| BearerSecret::parse(value.to_owned()).ok())
            .is_some_and(|bearer| self.credential.matches(&bearer))
    }
}
#[tonic::async_trait]
impl server_tools_server::ServerTools for Control {
    type InvokeStream = tokio_stream::Empty<Result<wire::InvocationObservation, Status>>;
    type PrepareExternalStream =
        tokio_stream::Iter<std::vec::IntoIter<Result<wire::ExternalCatalogFrame, Status>>>;

    async fn catalog(
        &self,
        request: Request<wire::CatalogRequest>,
    ) -> Result<Response<wire::CatalogResponse>, Status> {
        if !self.authorized(&request) {
            return Err(Status::unauthenticated("Expected fixture route"));
        }
        Ok(Response::new(wire::CatalogResponse {
            catalog_json: br#"{"tools":[{"name":"send","inputSchema":{"type":"object"}}]}"#
                .to_vec(),
        }))
    }
    async fn invoke(
        &self,
        _: Request<tonic::Streaming<wire::InvocationControl>>,
    ) -> Result<Response<Self::InvokeStream>, Status> {
        Err(Status::unimplemented("Not exercised by this fixture"))
    }
    async fn confirm_external_reuse(
        &self,
        _: Request<wire::ExternalReuse>,
    ) -> Result<Response<wire::ExternalReuseObservation>, Status> {
        Err(Status::unimplemented("Not exercised by this fixture"))
    }
    async fn prepare_external(
        &self,
        request: Request<wire::ExternalPreparation>,
    ) -> Result<Response<Self::PrepareExternalStream>, Status> {
        use wire::external_catalog_frame::Payload;
        if !self.authorized(&request) {
            return Err(Status::unauthenticated("Expected fixture route"));
        }
        let request = request.into_inner();
        if request.scope.as_ref().is_none_or(|scope| {
            scope.agent_id != "agent"
                || scope.conversation_id != "conversation"
                || scope
                    .source
                    .as_ref()
                    .is_none_or(|source| source.connection_id != "first")
        }) || request.operation_id != "setup"
            || request.execution_id != "execution"
        {
            return Err(Status::permission_denied("Expected fixture preparation"));
        }
        self.preparing.add_permits(1);
        self.prepare_release
            .acquire()
            .await
            .map_err(|_| Status::cancelled("Fixture closing"))?
            .forget();
        let description = "x".repeat(80_000);
        let catalogue = serde_json::to_vec(&serde_json::json!({
            "tools": [{"name":"lookup", "description":description, "inputSchema":{"type":"object"}}]
        }))
        .map_err(|_| Status::internal("Could not encode fixture"))?;
        let mut frames = vec![Ok(wire::ExternalCatalogFrame {
            integration_id: "unavailable".into(),
            payload: Some(Payload::Unavailable(
                wire::ExternalCatalogFailure::Timeout.into(),
            )),
        })];
        let parts = catalogue.chunks(48 * 1024);
        let count = parts.len();
        frames.extend(parts.enumerate().map(|(index, bytes)| {
            Ok(wire::ExternalCatalogFrame {
                integration_id: "lookup-server".into(),
                payload: Some(Payload::Catalogue(wire::ExternalCatalogChunk {
                    json: bytes.to_vec(),
                    final_chunk: index + 1 == count,
                    model_slot: 7,
                })),
            })
        }));
        frames.push(Ok(wire::ExternalCatalogFrame {
            integration_id: String::new(),
            payload: Some(Payload::End(wire::ExternalCatalogEnd {
                capacity_limited: false,
            })),
        }));
        Ok(Response::new(tokio_stream::iter(frames)))
    }
    async fn invoke_external(
        &self,
        request: Request<wire::ExternalInvocation>,
    ) -> Result<Response<wire::Final>, Status> {
        use wire::r#final::Outcome;
        if !self.authorized(&request) {
            return Err(Status::unauthenticated("Expected fixture route"));
        }
        let call = request.into_inner();
        let args = call.arguments_json.clone();
        self.entered
            .send(call)
            .await
            .map_err(|_| Status::cancelled("Fixture closed"))?;
        let outcome = match args.as_slice() {
            br#"{"case":"held"}"# => {
                self.release
                    .acquire()
                    .await
                    .map_err(|_| Status::cancelled("Fixture closed"))?
                    .forget();
                Outcome::ResultJson(br#"{"content":[],"structuredContent":{"proof":42}}"#.to_vec())
            }
            br#"{"case":"unknown"}"# => return Err(Status::unavailable("Fixture reply lost")),
            _ => Outcome::NotExecuted(wire::Rejection::Authority.into()),
        };
        Ok(Response::new(wire::Final {
            outcome: Some(outcome),
        }))
    }
}

#[tokio::test]
async fn external_http_mcp_uses_exact_component_scope_and_does_not_latch_reply_loss()
-> Result<(), AppError> {
    crate::test_support::scope(async |fixture| {
        let key = BearerSecret::generate()?;
        let (entered, mut calls) = mpsc::channel(4);
        let peer = Arc::new(Control {
            credential: key.digest(), entered, release: Semaphore::new(0),
            preparing: Semaphore::new(0), prepare_release: Semaphore::new(0),
        });
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let stop = fixture.shutdown();
        let component = peer.clone();
        fixture.server(async move {
            tonic::transport::Server::builder()
                .add_service(server_tools_server::ServerToolsServer::from_arc(component.clone()))
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
                    let _ = stop.await;
                    component.release.close();
                    component.prepare_release.close();
                }).await?;
            Ok::<_, AppError>(())
        });
        let channel = tonic::transport::Endpoint::from_shared(format!("http://{address}"))?
            .connect().await?;
        let route = CurrentRoute::default();
        let source = CaptureSource {
            session_id: "session".into(),
            control_id: "control".into(),
            connection_id: "first".into(),
        };
        route.update(channel.clone(), source.clone(), &key)?;
        let relay = Arc::new(tokio::sync::Mutex::new(Relay::start_builtin(
            route.clone(),
            Recipient { agent_id: "agent".into(), conversation_id: "conversation".into() },
            super::super::Ordering::default().open(binding()),
        ).await?));
        let cleanup = relay.clone();
        let release = peer.clone();
        fixture.cleanup(async move {
            release.release.close();
            release.prepare_release.close();
            cleanup.lock().await.finish().await
        });
        let prepare = PreparedExternal::fetch(
            &route, source.clone(), "agent".into(), "conversation".into(),
            "execution".into(), "setup".into(),
        );
        let builtin_ready = async {
            peer.preparing.acquire().await?.forget();
            let (url, bearer) = {
                let relay = relay.lock().await;
                assert_eq!(relay.optional_credentials().count(), 0);
                (relay.url().to_owned(), relay.bearer().to_owned())
            };
            let mut config = StreamableHttpClientTransportConfig::with_uri(url);
            config.auth_header = Some(bearer);
            let builtin = Profile.serve(StreamableHttpClientTransport::with_client(http_client()?, config)).await?;
            let tools = builtin.list_all_tools().await?;
            assert_eq!(tools.len(), 1);
            assert_eq!(tools[0].name, "send");
            relay.lock().await.verify().await?;
            builtin.cancel().await?;
            peer.prepare_release.add_permits(1);
            Ok::<_, AppError>(())
        };
        let (prepared, ()) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::try_join!(prepare, builtin_ready)
        }).await??;
        assert_eq!(prepared.entries.len(), 1, "Only complete usable lists are projected");
        let selection = prepared.selection.clone().ok_or("Missing preparation")?;
        relay.lock().await.install_optional(&prepared);
        let target = NativeTextTarget {
            snapshot: SnapshotId::parse("snapshot".into())?,
            native_session: NativeConversationId::parse("native".into())?,
        };
        let (url, bearer) = {
            let relay = relay.lock().await;
            relay.bind_external(target.clone(), selection.clone());
            let (_, url, bearer) = relay.optional_credentials().next().ok_or("Missing optional endpoint")?;
            (url, bearer.to_owned())
        };
        let mut config = StreamableHttpClientTransportConfig::with_uri(url);
        config.auth_header = Some(bearer);
        let client = Profile.serve(StreamableHttpClientTransport::with_client(http_client()?, config)).await?;
        let listed = client.list_all_tools().await?;
        assert_eq!(listed[0].description.as_ref().map(|text| text.len()), Some(80_000));
        let args = |case| -> Result<_, serde_json::Error> {
            Ok(CallToolRequestParams::new("lookup")
                .with_arguments(serde_json::from_value(serde_json::json!({"case":case}))?))
        };
        let held = args("held")?;
        let check = async {
            let call = calls.recv().await.ok_or("Missing relayed call")?;
            assert_eq!(call.integration_id, "lookup-server");
            assert_eq!(call.name, "lookup");
            let scope = call.preparation.ok_or("Missing preparation scope")?;
            assert_eq!(scope.operation_id, "setup");
            assert_eq!(scope.execution_id, "execution");
            assert_eq!(scope.scope.and_then(|value| value.source), Some(source.clone()));
            let native = call.target.ok_or("Missing native target")?;
            assert_eq!(native.snapshot_id, "snapshot");
            assert_eq!(native.native_session_id, "native");
            assert!(!relay.lock().await.quiescent());
            peer.release.add_permits(1);
            Ok::<_, AppError>(())
        };
        let (first, checked) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(client.call_tool(held), check)
        }).await?;
        checked?;
        let first = first?;
        assert_eq!(first.structured_content, Some(serde_json::json!({"proof":42})));

        let prepared_route = route.setup_route(&source).ok_or("Missing setup route")?;
        let successor = CaptureSource { connection_id: "successor".into(), ..source.clone() };
        route.update(channel, successor.clone(), &key)?;
        assert!(!prepared_route.is_open());
        assert!(route.setup_route(&source).is_none());
        assert!(route.setup_route(&successor).is_some_and(|route| route.is_open()));
        let stale = client.call_tool(args("unknown")?).await;
        assert!(matches!(stale, Err(rmcp::ServiceError::McpError(error)) if error.message.contains("invocation was not executed")));
        assert!(calls.try_recv().is_err());
        relay.lock().await.bind_external(target, chapchoo_harness::NativeExternalPreparation {
            connection_id: successor.connection_id.clone(), operation_id: "successor-setup".into(), ..selection
        });
        let lost = client.call_tool(args("unknown")?).await;
        assert!(matches!(lost, Err(rmcp::ServiceError::McpError(error)) if error.message.contains("effect may have occurred")));
        let current = calls.recv().await.ok_or("Missing successor invocation")?.preparation.ok_or("Missing successor scope")?;
        assert_eq!(current.operation_id, "successor-setup");
        assert_eq!(current.scope.and_then(|value| value.source), Some(successor));
        assert!(calls.try_recv().is_err(), "No repeated external request");
        assert!(!relay.lock().await.state.uncertain_effects.load(Ordering::Acquire));
        client.cancel().await?;
        Ok(())
    }).await
}
