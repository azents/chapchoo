//! Independently authenticated execution connectivity; never substrate power proof.
mod capture_sync;
mod operations;
use crate::AppError;
use chapchoo_crypto::BearerSecret;
use chapchoo_protocol::control::{
    self as wire, connector_authentication_client::ConnectorAuthenticationClient,
    connector_connection_client::ConnectorConnectionClient,
};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use tonic::{
    Request, Status,
    transport::{Channel, Endpoint},
};

fn request<T>(value: T, key: &BearerSecret) -> Result<Request<T>, AppError> {
    let mut request = Request::new(value);
    request
        .metadata_mut()
        .insert("authorization", format!("Bearer {}", key.expose()).parse()?);
    request.set_timeout(Duration::from_secs(10));
    Ok(request)
}
fn mutation_id() -> Result<String, AppError> {
    Ok(BearerSecret::generate()?
        .digest()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
fn endpoint(value: &str) -> Result<Endpoint, AppError> {
    let parsed = url::Url::parse(value)?;
    if parsed.scheme() != "http"
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || parsed.path() != "/"
    {
        return Err(
            "Connector requires an explicit trusted-network HTTP/2 Control endpoint".into(),
        );
    }
    Ok(Endpoint::from_shared(value.to_owned())?
        .connect_timeout(Duration::from_secs(5))
        .buffer_size(8)
        .concurrency_limit(8))
}
/// This process owns one execution-scoped Connector session. The initial key is
/// supplied by the execution's dedicated bootstrap Secret, never a Provider key.
pub async fn run(
    endpoint_url: &str,
    session_id: String,
    initial_key: BearerSecret,
    stop: CancellationToken,
    native: chapchoo_harness::NativeOwner,
) -> Result<(), AppError> {
    // This owner survives every operational key/stream replacement. Reads and
    // connectivity do not start native work; source-qualified commands do later.
    let mut capture_sync = capture_sync::CaptureSync::new(native.handle());
    let mut operations = operations::Operations::new(native.handle());
    let primary = reconnect(
        endpoint_url,
        session_id,
        initial_key,
        stop,
        &mut capture_sync,
        &mut operations,
    )
    .await;
    let (capture_cleanup, operation_cleanup, cleanup) =
        tokio::join!(capture_sync.finish(), operations.finish(), native.finish());
    let primary = crate::completion::complete(primary, capture_cleanup);
    let primary = crate::completion::complete(primary, operation_cleanup);
    match (primary, cleanup) {
        (Err(primary), Err(cleanup)) => Err(ConnectorCleanupFailure { primary, cleanup }.into()),
        (Err(primary), Ok(_)) => Err(primary),
        (Ok(()), Err(cleanup)) => Err(cleanup.into()),
        (Ok(()), Ok(_)) => Ok(()),
    }
}
struct ConnectorCleanupFailure {
    primary: AppError,
    cleanup: chapchoo_harness::Error,
}
impl std::fmt::Debug for ConnectorCleanupFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectorCleanupFailure")
            .field("cleanup", &self.cleanup)
            .finish_non_exhaustive()
    }
}
impl std::fmt::Display for ConnectorCleanupFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Connector failed and native retirement is unconfirmed")
    }
}
impl std::error::Error for ConnectorCleanupFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.primary.as_ref())
    }
}
async fn reconnect(
    endpoint_url: &str,
    session_id: String,
    initial_key: BearerSecret,
    stop: CancellationToken,
    capture_sync: &mut capture_sync::CaptureSync,
    operations: &mut operations::Operations,
) -> Result<(), AppError> {
    let channel = endpoint(endpoint_url)?.connect_lazy();
    let mut authentication = ConnectorAuthenticationClient::new(channel.clone())
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024);
    // Credential mutation retries, if the caller starts a new process, use the
    // execution bootstrap recovery contract. Never replay a machine effect here.
    let exchanged = tokio::select! {
        _=stop.cancelled()=>return Ok(()),
        result=authentication.exchange(request(wire::ConnectorKeyRequest{session_id:session_id.clone(),request_id:mutation_id()?},&initial_key)?)=>result?.into_inner(),
    };
    let mut key = BearerSecret::parse(exchanged.token)?;
    let mut proactive =
        tokio::time::Instant::now() + refresh_delay(exchanged.proactive_after.as_ref())?;
    loop {
        let outcome = connected(
            channel.clone(),
            &mut authentication,
            &session_id,
            &mut key,
            &mut proactive,
            &stop,
            (capture_sync, operations),
        )
        .await;
        if let Err(error) = outcome
            && (error.downcast_ref::<tokio::task::JoinError>().is_some()
                || error.downcast_ref::<Status>().is_some_and(|e| {
                    matches!(
                        e.code(),
                        tonic::Code::Unauthenticated | tonic::Code::PermissionDenied
                    )
                }))
        {
            return Err(error);
        }
        if stop.is_cancelled() {
            return Ok(());
        }
        // Reconnect is a new stream admission, never resumption of old calls.
        tokio::select! {_=stop.cancelled()=>return Ok(()),_=tokio::time::sleep(Duration::from_secs(1))=>{}}
    }
}
fn refresh_delay(value: Option<&prost_types::Timestamp>) -> Result<Duration, AppError> {
    let value = value.ok_or("Credential refresh time is missing")?;
    let target = i128::from(value.seconds) * 1000 + i128::from(value.nanos) / 1_000_000;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis();
    Ok(Duration::from_millis(u64::try_from(
        (target - i128::try_from(now)?).max(0),
    )?))
}
async fn connected(
    channel: Channel,
    authentication: &mut ConnectorAuthenticationClient<Channel>,
    session: &str,
    key: &mut BearerSecret,
    proactive: &mut tokio::time::Instant,
    stop: &CancellationToken,
    workers: (&mut capture_sync::CaptureSync, &mut operations::Operations),
) -> Result<(), AppError> {
    let (capture_sync, operations) = workers;
    let (sender, receiver) = mpsc::channel(8);
    sender
        .send(wire::ConnectorFrame {
            payload: Some(wire::connector_frame::Payload::Hello(
                wire::ConnectorHello {
                    session_id: session.into(),
                },
            )),
        })
        .await?;
    let mut client = ConnectorConnectionClient::new(channel.clone())
        .max_decoding_message_size(64 * 1024)
        .max_encoding_message_size(64 * 1024);
    // Streaming lifetime has no unary ten-second timeout. Hello and individual
    // command/auth operations retain bounded deadlines.
    let mut open = Request::new(ReceiverStream::new(receiver));
    open.metadata_mut()
        .insert("authorization", format!("Bearer {}", key.expose()).parse()?);
    let mut stream = tokio::time::timeout(Duration::from_secs(10), client.open(open))
        .await??
        .into_inner();
    let first = tokio::time::timeout(Duration::from_secs(10), stream.message())
        .await??
        .ok_or("Connector acceptance is missing")?;
    let Some(wire::control_frame::Payload::Connected(connected)) = first.payload else {
        return Err("Expected Connector acceptance".into());
    };
    if connected.heartbeat_interval_ms == 0
        || connected.validity_ms <= connected.heartbeat_interval_ms
    {
        return Err("Connector heartbeat policy is invalid".into());
    }
    let mut heartbeat = tokio::time::interval(Duration::from_millis(u64::from(
        connected.heartbeat_interval_ms,
    )));
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let renewal = tokio::time::sleep_until(*proactive);
    tokio::pin!(renewal);
    let mut last_heartbeat = tokio::time::Instant::now();
    let mut capture_tick = tokio::time::interval(Duration::from_millis(500));
    capture_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _=stop.cancelled()=>return Ok(()),
            _=capture_tick.tick()=>{
                capture_sync.schedule(channel.clone(),chapchoo_protocol::conversations::CaptureSource {
                    session_id:session.into(),control_id:connected.instance_id.clone(),connection_id:connected.connection_id.clone(),
                },key)?;
                operations.publish(channel.clone(),chapchoo_protocol::conversations::CaptureSource {
                    session_id:session.into(),control_id:connected.instance_id.clone(),connection_id:connected.connection_id.clone(),
                },key)?;
            }
            _=&mut renewal=>{
                let result=authentication.refresh(request(wire::ConnectorKeyRequest{session_id:session.into(),request_id:mutation_id()?},key)?).await?.into_inner();
                *key=BearerSecret::parse(result.token)?;
                *proactive=tokio::time::Instant::now()+refresh_delay(result.proactive_after.as_ref())?;
                // The route was admitted with the old key. Reconnect with the new
                // credential before its bounded overlap expires.
                return Ok(());
            }
            _=heartbeat.tick()=>{
                if last_heartbeat.elapsed()>=Duration::from_millis(u64::from(connected.validity_ms)){return Err("Connector heartbeat acknowledgement expired".into());}
                sender.try_send(wire::ConnectorFrame{payload:Some(wire::connector_frame::Payload::Heartbeat(wire::Heartbeat{connection_id:connected.connection_id.clone()}))})?;
            }
            frame=stream.message()=>{
                let frame=frame?.ok_or("Connector stream closed")?;
                match frame.payload {
                    Some(wire::control_frame::Payload::Heartbeat(heartbeat)) if heartbeat.connection_id==connected.connection_id=>last_heartbeat=tokio::time::Instant::now(),
                    Some(wire::control_frame::Payload::Connector(command))=>{
                        if command.remaining_ms==0||command.identity.as_ref().is_none_or(|id|id.connection_id!=connected.connection_id){continue;}
                        let mut check=request(wire::ConnectorCommandCheck{source:Some(wire::ConnectorConnectionCheck{session_id:session.into(),instance_id:connected.instance_id.clone(),connection_id:connected.connection_id.clone()}),command:Some(command.clone())},key)?;
                        check.set_timeout(Duration::from_millis(u64::from(command.remaining_ms.min(10_000))));
                        if authentication.check_command(check).await.is_err(){continue;}
                        let context=command.computer.ok_or("Connector Computer context is missing")?;
                        let admitted=match command.operation {
                            Some(wire::connector_command::Operation::Observe(_))=>None,
                            Some(wire::connector_command::Operation::Stop(stop_command))=>{
                                let mut client=chapchoo_protocol::conversations::connector_capture_client::ConnectorCaptureClient::new(channel.clone())
                                    .max_decoding_message_size(64*1024).max_encoding_message_size(64*1024);
                                let check=request(chapchoo_protocol::conversations::StopCheck {source:Some(chapchoo_protocol::conversations::CaptureSource {session_id:session.into(),control_id:connected.instance_id.clone(),connection_id:connected.connection_id.clone()}),command:Some(stop_command.clone())},key)?;
                                if client.check_stop(check).await.is_err() {Some(false)} else {Some(operations.stop(stop_command)?)}
                            },
                            Some(wire::connector_command::Operation::Answer(answer))=>Some(operations.answer(answer,channel.clone(),chapchoo_protocol::conversations::CaptureSource {
                                session_id:session.into(),control_id:connected.instance_id.clone(),connection_id:connected.connection_id.clone(),
                            },key)?),
                            Some(wire::connector_command::Operation::Input(input))=>Some(operations.input(input,channel.clone(),chapchoo_protocol::conversations::CaptureSource {
                                session_id:session.into(),control_id:connected.instance_id.clone(),connection_id:connected.connection_id.clone(),
                            },key)?),
                            Some(wire::connector_command::Operation::Setup(setup))=>{
                                let mut client=chapchoo_protocol::conversations::connector_capture_client::ConnectorCaptureClient::new(channel.clone())
                                    .max_decoding_message_size(64*1024).max_encoding_message_size(64*1024);
                                let check=request(chapchoo_protocol::conversations::SetupCheck {
                                    source:Some(chapchoo_protocol::conversations::CaptureSource {session_id:session.into(),control_id:connected.instance_id.clone(),connection_id:connected.connection_id.clone()}),
                                    command:Some(setup.clone()),
                                },key)?;
                                if client.check_setup(check).await.is_err() {Some(false)} else {Some(operations.setup(setup)?)}
                            },
                            None=>continue,
                        };
                        sender.try_send(wire::ConnectorFrame{payload:Some(wire::connector_frame::Payload::Result(wire::ConnectorResult{identity:command.identity,execution_id:context.execution_id,ready:true,admitted}))})?;
                    }
                    Some(wire::control_frame::Payload::Cancel(_))=>{},
                    _=>return Err("Unexpected Connector frame".into()),
                }
            }
        }
    }
}
