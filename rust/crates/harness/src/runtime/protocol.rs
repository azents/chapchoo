use super::*;
#[cfg(test)]
mod tests;

#[cfg(test)]
async fn connect(
    transport: impl agent_client_protocol::ConnectTo<Client>,
    requests: Requests,
    observations: Arc<Observations>,
    ready: oneshot::Sender<Result<agent_client_protocol::v2::ActiveSession<Agent>, Error>>,
    done: CancellationToken,
    cwd: std::path::PathBuf,
    context: NativeContext,
) -> Result<(), agent_client_protocol::Error> {
    connect_authenticated(
        transport,
        requests,
        observations,
        ready,
        done,
        cwd,
        (context, None),
    )
    .await
}
pub(super) async fn connect_authenticated(
    transport: impl agent_client_protocol::ConnectTo<Client>,
    requests: Requests,
    observations: Arc<Observations>,
    ready: oneshot::Sender<Result<agent_client_protocol::v2::ActiveSession<Agent>, Error>>,
    done: CancellationToken,
    cwd: std::path::PathBuf,
    setup: (NativeContext, Option<NativeCredentials>),
) -> Result<(), agent_client_protocol::Error> {
    let permission = requests.clone();
    let forms = requests;
    let permission_observation = observations.clone();
    let form_observation = observations;
    Client
        .v2()
        .on_receive_notification(
            async |_update: schema::UpdateSessionNotification, _cx: ConnectionTo<Agent>| Ok(()),
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_request(
            async move |request: schema::RequestPermissionRequest,
                        responder: Responder<schema::RequestPermissionResponse>,
                        _cx: ConnectionTo<Agent>| {
                let mut pending = permission.lock().await;
                pending.retain(|_, (_, r)| !r.cancelled());
                let position = permission_observation
                    .take_request(responder.id())
                    .map_err(|_| agent_client_protocol::Error::internal_error())?;
                if !requests::supported_permission(&request) {
                    return responder
                        .respond_with_error(agent_client_protocol::Error::invalid_params());
                }
                let key = observations::token("request")
                    .map_err(|_| agent_client_protocol::Error::internal_error())?;
                if pending.len() >= MAX_REQUESTS {
                    return responder
                        .respond_with_error(agent_client_protocol::Error::internal_error());
                }
                pending.insert(key, (position, Pending::Permission(request, responder)));
                permission_observation.changed();
                Ok(())
            },
            agent_client_protocol::on_receive_request!(),
        )
        .on_receive_request(
            async move |request: schema::CreateElicitationRequest,
                        responder: Responder<schema::CreateElicitationResponse>,
                        _cx: ConnectionTo<Agent>| {
                let mut pending = forms.lock().await;
                pending.retain(|_, (_, r)| !r.cancelled());
                let position = form_observation
                    .take_request(responder.id())
                    .map_err(|_| agent_client_protocol::Error::internal_error())?;
                if !requests::supported_form(&request) {
                    return responder
                        .respond_with_error(agent_client_protocol::Error::invalid_params());
                }
                let key = observations::token("request")
                    .map_err(|_| agent_client_protocol::Error::internal_error())?;
                if pending.len() >= MAX_REQUESTS {
                    return responder
                        .respond_with_error(agent_client_protocol::Error::internal_error());
                }
                pending.insert(key, (position, Pending::Form(request, responder)));
                form_observation.changed();
                Ok(())
            },
            agent_client_protocol::on_receive_request!(),
        )
        .connect_with(transport, async move |cx: ConnectionTo<Agent>| {
            let session = match initialize_session(cx, cwd, setup).await {
                Ok(session) => session,
                Err((stage, cause)) => {
                    let _ = ready.send(Err(Error::ProtocolRejected {
                        stage,
                        code: i32::from(cause.code),
                    }));
                    return Err(cause);
                }
            };
            let _ = ready.send(Ok(session));
            done.cancelled().await;
            Ok(())
        })
        .await
}

async fn initialize_session(
    cx: ConnectionTo<Agent>,
    cwd: std::path::PathBuf,
    setup: (NativeContext, Option<NativeCredentials>),
) -> Result<
    agent_client_protocol::v2::ActiveSession<Agent>,
    (crate::ProtocolStage, agent_client_protocol::Error),
> {
    use crate::ProtocolStage;
    let (context, credentials) = setup;
    let initialize = schema::InitializeRequest::new(
        ProtocolVersion::V2,
        schema::Implementation::new("chapchoo", env!("CARGO_PKG_VERSION")),
    )
    .capabilities(schema::ClientCapabilities::new().elicitation(
        schema::ElicitationCapabilities::new().form(schema::ElicitationFormCapabilities::new()),
    ));
    let initialized = cx
        .send_initialize(initialize)
        .block_task()
        .await
        .map_err(|e| (ProtocolStage::Initialize, e))?;
    let agent_client_protocol::v2::InitializeOutcome::Initialized(response) = initialized else {
        return Err((
            ProtocolStage::Initialize,
            agent_client_protocol::Error::invalid_params(),
        ));
    };
    if response.protocol_version != ProtocolVersion::V2 || response.capabilities.session.is_none() {
        return Err((
            ProtocolStage::Initialize,
            agent_client_protocol::Error::invalid_params(),
        ));
    }
    if let Some(key) = credentials
        .as_ref()
        .and_then(NativeCredentials::initial_api_key)
    {
        let mut meta = serde_json::Map::new();
        meta.insert("api-key".into(), serde_json::json!({"apiKey":key}));
        cx.send_request(schema::LoginAuthRequest::new("api-key").meta(meta))
            .block_task()
            .await
            .map_err(|error| (ProtocolStage::Authentication, error))?;
    }
    // Credentials never enter capture. Do not install an external-token callback
    // or an interactive/native-login fallback.
    drop(credentials);
    let session = match context {
        NativeContext::Resume {
            session: id,
            replay,
        } => {
            let request = schema::ResumeSessionRequest::new(id.as_str(), cwd);
            let request = match replay {
                CaptureReplay::FromStart => {
                    request.replay_from(schema::ReplayFrom::Start(schema::ReplayFromStart::new()))
                }
                CaptureReplay::AlreadyCaptured => request,
            };
            cx.resume_session_from(request)
                .start_session()
                .block_task()
                .await
                .map_err(|e| (ProtocolStage::Session, e))?
                .session
        }
        NativeContext::New => {
            cx.build_session(cwd)
                .start_session()
                .block_task()
                .await
                .map_err(|e| (ProtocolStage::Session, e))?
                .session
        }
    };
    Ok(session)
}
