//! Dedicated bounded component tool streams, never Control command frames.
use crate::{ingress, status::status};
use chapchoo_crypto::SecretDigest;
use chapchoo_protocol::server_tools::{
    self as wire,
    server_tools_server::{ServerTools as WireTools, ServerToolsServer},
};
use chapchoo_service::{
    DomainReason, ServerToolObservation, ServerToolOutcome, ServerToolRejection, ServerToolRequest,
    ServerToolScope, ServerTools, ServiceError,
};
use prost::Message;
use std::{
    error::Error,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::{Mutex, Semaphore, mpsc},
    task::{JoinError, JoinSet},
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

#[derive(Clone)]
pub struct ServerToolServices {
    owner: ServerTools,
    admission: Arc<AtomicBool>,
    ingress: Arc<Semaphore>,
    streams: Arc<Mutex<JoinSet<()>>>,
    failures: Arc<std::sync::Mutex<Vec<JoinError>>>,
}
impl fmt::Debug for ServerToolServices {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ServerToolServices([REDACTED])")
    }
}
impl ServerToolServices {
    pub fn new(owner: ServerTools, admission: Arc<AtomicBool>) -> Self {
        Self {
            owner,
            admission,
            ingress: Arc::new(Semaphore::new(144)),
            streams: Arc::new(Mutex::new(JoinSet::new())),
            failures: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
    pub fn server(&self) -> ServerToolsServer<Self> {
        ServerToolsServer::new(self.clone())
            .max_decoding_message_size(64 * 1024)
            .max_encoding_message_size(64 * 1024)
    }
    fn admit(&self) -> Result<(), Status> {
        if self.admission.load(Ordering::Acquire) {
            Ok(())
        } else {
            Err(Status::unavailable("Server tools are draining"))
        }
    }
    fn collect(&self, result: Result<(), JoinError>) {
        if let Err(error) = result {
            self.failures
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(error);
        }
    }
    /// Composition closes shared admission first. Stream forwarding and the
    /// effect owner drain together; cancelling this waiter retains both JoinSets.
    pub async fn finish(&self) -> Result<(), ServiceError> {
        self.owner.close();
        let forwarding = async {
            let mut streams = self.streams.lock().await;
            while let Some(result) = streams.join_next().await {
                self.collect(result);
            }
        };
        let (effect, ()) = tokio::join!(self.owner.finish(), forwarding);
        #[derive(Debug)]
        struct Failures {
            effect: Option<ServiceError>,
            streams: Vec<JoinError>,
        }
        impl fmt::Display for Failures {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "Server tools failed to drain: effect={}, streams={}",
                    self.effect.is_some(),
                    self.streams.len()
                )
            }
        }
        impl Error for Failures {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                self.effect
                    .as_ref()
                    .map(|error| error as &(dyn Error + 'static))
                    .or_else(|| {
                        self.streams
                            .first()
                            .map(|error| error as &(dyn Error + 'static))
                    })
            }
        }
        let streams = std::mem::take(
            &mut *self
                .failures
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        if effect.is_ok() && streams.is_empty() {
            Ok(())
        } else {
            Err(ServiceError::infrastructure(Failures {
                effect: effect.err(),
                streams,
            }))
        }
    }
}
fn scope(
    value: Option<wire::CallerScope>,
    credential: SecretDigest,
) -> Result<ServerToolScope, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("Tool caller scope is required"))?;
    let source = value
        .source
        .ok_or_else(|| Status::invalid_argument("Tool component source is required"))?;
    for id in [
        &source.session_id,
        &source.control_id,
        &source.connection_id,
        &value.agent_id,
        &value.conversation_id,
    ] {
        ingress::id(id)?;
    }
    Ok(ServerToolScope {
        session_id: source.session_id,
        control_id: source.control_id,
        connection_id: source.connection_id,
        credential,
        agent_id: value.agent_id,
        conversation_id: value.conversation_id,
    })
}
fn observation(value: ServerToolObservation) -> wire::InvocationObservation {
    use wire::invocation_observation::Observation;
    let value = match value {
        ServerToolObservation::Waiting => Observation::Waiting(()),
        ServerToolObservation::Started => Observation::Started(()),
        ServerToolObservation::Final(outcome) => {
            use wire::r#final::Outcome;
            let outcome = match outcome {
                ServerToolOutcome::Result(bytes) => Outcome::ResultJson(bytes),
                ServerToolOutcome::Uncertain => Outcome::Uncertain(()),
                ServerToolOutcome::UncertainRepresentation => Outcome::UncertainRepresentation(()),
                ServerToolOutcome::NotExecuted(reason) => {
                    let reason = match reason {
                        ServerToolRejection::InvalidEnvelope => wire::Rejection::InvalidEnvelope,
                        ServerToolRejection::UnknownTool => wire::Rejection::UnknownTool,
                        ServerToolRejection::Capacity => wire::Rejection::Capacity,
                        ServerToolRejection::Closing => wire::Rejection::Closing,
                        ServerToolRejection::Cancelled => wire::Rejection::Cancelled,
                        ServerToolRejection::QueueExpired => wire::Rejection::QueueExpired,
                        ServerToolRejection::ResponseExpired => wire::Rejection::ResponseExpired,
                        ServerToolRejection::Authority => wire::Rejection::Authority,
                    };
                    Outcome::NotExecuted(reason as i32)
                }
            };
            Observation::Final(wire::Final {
                outcome: Some(outcome),
            })
        }
    };
    wire::InvocationObservation {
        observation: Some(value),
    }
}

/// Only the opening boundary may assert known nonexecution. After submit
/// succeeds, ordinary status/stream loss retains possible-effect meaning.
fn not_executed(mut status: Status, reason: wire::Rejection) -> Status {
    status.metadata_mut().insert(
        wire::NOT_EXECUTED_METADATA,
        tonic::metadata::MetadataValue::from_static(reason.as_str_name()),
    );
    status
}

#[tonic::async_trait]
impl WireTools for ServerToolServices {
    async fn catalog(
        &self,
        request: Request<wire::CatalogRequest>,
    ) -> Result<Response<wire::CatalogResponse>, Status> {
        self.admit()?;
        let _permit = self
            .ingress
            .clone()
            .try_acquire_owned()
            .map_err(|_| Status::resource_exhausted("Server tool ingress is full"))?;
        let credential = ingress::actor(&request)?;
        let scope = scope(request.into_inner().scope, credential)?;
        let catalog_json = self.owner.catalog(scope).await.map_err(status)?;
        Ok(Response::new(wire::CatalogResponse { catalog_json }))
    }
    type InvokeStream = ReceiverStream<Result<wire::InvocationObservation, Status>>;
    async fn invoke(
        &self,
        request: Request<Streaming<wire::InvocationControl>>,
    ) -> Result<Response<Self::InvokeStream>, Status> {
        use wire::Rejection as R;
        self.admit()
            .map_err(|error| not_executed(error, R::Closing))?;
        let permit = self.ingress.clone().try_acquire_owned().map_err(|_| {
            not_executed(
                Status::resource_exhausted("Server tool ingress is full"),
                R::Capacity,
            )
        })?;
        let credential =
            ingress::actor(&request).map_err(|error| not_executed(error, R::Authority))?;
        let mut input = request.into_inner();
        let first = tokio::time::timeout(Duration::from_secs(10), input.message())
            .await
            .map_err(|_| {
                not_executed(
                    Status::deadline_exceeded("Tool Submit did not arrive"),
                    R::ResponseExpired,
                )
            })?
            .map_err(|error| not_executed(error, R::InvalidEnvelope))?
            .ok_or_else(|| {
                not_executed(
                    Status::invalid_argument("Tool Submit is required"),
                    R::InvalidEnvelope,
                )
            })?;
        if first.encoded_len() > 64 * 1024 {
            return Err(not_executed(
                Status::resource_exhausted("Tool Submit is too large"),
                R::InvalidEnvelope,
            ));
        }
        let Some(wire::invocation_control::Control::Submit(submit)) = first.control else {
            return Err(not_executed(
                Status::invalid_argument("The first tool frame must be Submit"),
                R::InvalidEnvelope,
            ));
        };
        let scope = scope(submit.scope, credential)
            .map_err(|error| not_executed(error, R::InvalidEnvelope))?;
        let mut streams = self.streams.try_lock().map_err(|_| {
            not_executed(
                Status::resource_exhausted("Server tools are busy"),
                R::Capacity,
            )
        })?;
        while let Some(result) = streams.try_join_next() {
            self.collect(result);
        }
        self.admit()
            .map_err(|error| not_executed(error, R::Closing))?;
        let mut invocation = self
            .owner
            .submit(ServerToolRequest {
                scope,
                name: submit.name,
                arguments: submit.arguments_json,
            })
            .map_err(|error| {
                let reason = match error.reason() {
                    DomainReason::InvalidInput => R::InvalidEnvelope,
                    DomainReason::CapacityExhausted => R::Capacity,
                    // No owner accepted the call, including worker failure.
                    _ => R::Closing,
                };
                not_executed(status(error), reason)
            })?;
        let control = invocation.control();
        let (sender, receiver) = mpsc::channel(3);
        streams.spawn(async move {
            let _permit = permit;
            let mut input_closed = false;
            let mut cancelled = false;
            loop {
                tokio::select! {
                    next = input.message(), if !input_closed => match next {
                        Ok(None) => input_closed = true,
                        Ok(Some(wire::InvocationControl {
                            control: Some(wire::invocation_control::Control::Cancel(())),
                        })) if !cancelled => {
                            cancelled = true;
                            control.cancel();
                        }
                        _ => {
                            // A malformed later frame or transport loss cannot
                            // establish rollback/non-execution of prior Submit.
                            let _ = sender.send(Err(Status::invalid_argument(
                                "Tool control ended; an earlier effect may have occurred"
                            ))).await;
                            break;
                        }
                    },
                    event = invocation.next() => {
                        let Some(event) = event else { break; };
                        let terminal = matches!(event, ServerToolObservation::Final(_));
                        if sender.send(Ok(observation(event))).await.is_err() || terminal { break; }
                    }
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(receiver)))
    }
}
