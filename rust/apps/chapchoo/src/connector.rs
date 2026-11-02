//! Independently authenticated execution connectivity; never substrate power proof.
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
        )
        .await;
        if stop.is_cancelled() {
            return Ok(());
        }
        if let Err(error) = outcome
            && error.downcast_ref::<Status>().is_some_and(|e| {
                matches!(
                    e.code(),
                    tonic::Code::Unauthenticated | tonic::Code::PermissionDenied
                )
            })
        {
            return Err(error);
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
) -> Result<(), AppError> {
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
    let mut client = ConnectorConnectionClient::new(channel)
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
    loop {
        tokio::select! {
            _=stop.cancelled()=>return Ok(()),
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
                        sender.try_send(wire::ConnectorFrame{payload:Some(wire::connector_frame::Payload::Result(wire::ConnectorResult{identity:command.identity,execution_id:context.execution_id,ready:true}))})?;
                    }
                    Some(wire::control_frame::Payload::Cancel(_))=>{},
                    _=>return Err("Unexpected Connector frame".into()),
                }
            }
        }
    }
}
