//! Local App Server JSONL: independent request ownership and ordered ingress.
use crate::Error;
use futures_util::{
    SinkExt, StreamExt,
    future::{AbortHandle, Abortable},
    stream::FuturesUnordered,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{mpsc, oneshot},
};
use tokio_util::{
    codec::{FramedRead, FramedWrite, LinesCodec},
    sync::CancellationToken,
};
use zeroize::Zeroizing;

const CAPACITY: usize = 32;

pub(super) struct Client {
    ids: AtomicU64,
    commands: mpsc::Sender<Outbound>,
}

struct Outbound {
    line: Zeroizing<String>,
    reply: Option<(u64, oneshot::Sender<Result<Value, i32>>)>,
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub(super) enum CallFailure {
    #[error("App Server explicitly rejected the request ({0})")]
    Rejected(i32),
    #[error("App Server disconnected before an attributed reply")]
    Uncertain,
}

pub(super) struct Reply(oneshot::Receiver<Result<Value, i32>>);

#[derive(Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(untagged)]
pub(super) enum RequestId {
    Number(i64),
    String(String),
}

pub(super) enum ServerReply {
    #[cfg(test)]
    Immediate(Value),
    Deferred(oneshot::Receiver<Result<Value, i32>>),
}

struct PendingServerRequest {
    generation: u64,
    thread: String,
    abort: AbortHandle,
}

impl Drop for PendingServerRequest {
    fn drop(&mut self) {
        self.abort.abort();
    }
}

impl Reply {
    pub(super) async fn receive(self) -> Result<Value, CallFailure> {
        self.0
            .await
            .map_err(|_| CallFailure::Uncertain)?
            .map_err(CallFailure::Rejected)
    }
}

impl Client {
    pub(super) fn is_closed(&self) -> bool {
        self.commands.is_closed()
    }

    /// A successful return means publication to this connection's unique
    /// writer, not native acceptance. Dropping Reply does not withdraw it.
    pub(super) fn enqueue(&self, method: &str, params: impl Serialize) -> Result<Reply, Error> {
        #[derive(Serialize)]
        struct Request<'a, P> {
            id: u64,
            method: &'a str,
            params: P,
        }
        let id = self
            .ids
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| Error::NotAdmitted)?;
        let (send, receive) = oneshot::channel();
        self.publish(Request { id, method, params }, Some((id, send)))?;
        Ok(Reply(receive))
    }

    pub(super) fn initialized(&self) -> Result<(), Error> {
        self.publish(json!({"method": "initialized"}), None)
    }

    fn publish(
        &self,
        frame: impl Serialize,
        reply: Option<(u64, oneshot::Sender<Result<Value, i32>>)>,
    ) -> Result<(), Error> {
        let slot = self
            .commands
            .try_reserve()
            .map_err(|_| Error::NotAdmitted)?;
        let line = Zeroizing::new(serde_json::to_string(&frame).map_err(|_| Error::Protocol)?);
        if line.len() > super::super::MAX_LINE {
            return Err(Error::NotAdmitted);
        }
        slot.send(Outbound { line, reply });
        Ok(())
    }
}

/// The caller owns the task returned by the driver and the physical child.
/// Non-reply messages reach the consumer synchronously, in wire order.
#[cfg(test)]
pub(super) fn connect(
    read: impl AsyncRead + Unpin,
    write: impl AsyncWrite + Unpin,
    done: CancellationToken,
    mut ingress: impl FnMut(Value) -> Result<Option<Value>, Error>,
) -> (Client, impl Future<Output = Result<(), Error>>) {
    connect_with_requests(read, write, done, move |_, message| {
        ingress(message).map(|reply| reply.map(ServerReply::Immediate))
    })
}

/// The ordered ingress callback may retain a responder without blocking
/// reads. It receives the actual wire bytes, not a reserialized envelope.
pub(super) fn connect_with_requests(
    read: impl AsyncRead + Unpin,
    write: impl AsyncWrite + Unpin,
    done: CancellationToken,
    ingress: impl FnMut(&str, Value) -> Result<Option<ServerReply>, Error>,
) -> (Client, impl Future<Output = Result<(), Error>>) {
    let (commands, outbound) = mpsc::channel(CAPACITY);
    (
        Client {
            ids: AtomicU64::new(1),
            commands,
        },
        drive(read, write, done, outbound, ingress),
    )
}

async fn drive(
    read: impl AsyncRead + Unpin,
    write: impl AsyncWrite + Unpin,
    done: CancellationToken,
    mut outbound: mpsc::Receiver<Outbound>,
    mut ingress: impl FnMut(&str, Value) -> Result<Option<ServerReply>, Error>,
) -> Result<(), Error> {
    let codec = || LinesCodec::new_with_max_length(super::super::MAX_LINE);
    let mut input = FramedRead::new(read, codec());
    let mut output = FramedWrite::new(write, codec());
    let mut pending = BTreeMap::new();
    let mut incoming = BTreeMap::<RequestId, PendingServerRequest>::new();
    let mut answers = FuturesUnordered::new();
    let mut generation = 0_u64;
    loop {
        tokio::select! {
            () = done.cancelled() => return Ok(()),
            command = outbound.recv(), if pending.len() < CAPACITY => {
                let Some(command) = command else { return Ok(()) };
                if let Some((id, reply)) = command.reply {
                    pending.insert(id, reply);
                }
                // Cancellation while writing leaves an uncertain result. The
                // owner must retire this process before admitting a successor.
                tokio::select! {
                    () = done.cancelled() => return Ok(()),
                    result = output.send(command.line.as_str()) => result.map_err(|_| Error::Protocol)?,
                }
            }
            line = input.next() => {
                let line = line.ok_or(Error::Protocol)?.map_err(|_| Error::Protocol)?;
                let message: Value = serde_json::from_str(&line).map_err(|_| Error::Protocol)?;
                if !message.is_object() {
                    return Err(Error::Protocol);
                }
                if message.get("method").is_some() {
                    if !message["method"].is_string() || message.get("result").is_some() || message.get("error").is_some() {
                        return Err(Error::Protocol);
                    }
                    let id = message.get("id").map(request_id).transpose()?;
                    if id.as_ref().is_some_and(|id| incoming.contains_key(id)) {
                        return Err(Error::Protocol);
                    }
                    if message["method"] == "serverRequest/resolved" {
                        if id.is_some() {
                            return Err(Error::Protocol);
                        }
                        let scope = &message["params"];
                        let resolved = request_id(&scope["requestId"])?;
                        if incoming.get(&resolved).is_some_and(|target| {
                            scope["threadId"].as_str() == Some(target.thread.as_str())
                        }) {
                            incoming.remove(&resolved);
                        }
                    }
                    // Reject over-capacity requests before allowing them to be
                    // captured as user-answerable. Do not drop an open request.
                    if let Some(id) = &id && answers.len() >= CAPACITY {
                        send_response(&mut output, &done, &error_response(id, -32603)).await?;
                        continue;
                    }
                    let thread = id.as_ref().and_then(|_| {
                        message["params"]["threadId"].as_str().filter(|id| super::valid_key(id)).map(str::to_owned)
                    });
                    match (id, ingress(&line, message)?) {
                        #[cfg(test)]
                        (Some(_), Some(ServerReply::Immediate(response))) => {
                            send_response(&mut output, &done, &response).await?;
                        }
                        (Some(id), Some(ServerReply::Deferred(answer))) => {
                            let thread = thread.ok_or(Error::Protocol)?;
                            generation = generation.checked_add(1).ok_or(Error::Protocol)?;
                            let epoch = generation;
                            let (abort, registration) = AbortHandle::new_pair();
                            incoming.insert(id.clone(), PendingServerRequest {
                                generation: epoch, thread, abort,
                            });
                            answers.push(async move {
                                (id, epoch, Abortable::new(answer, registration).await)
                            });
                        }
                        (Some(id), None) => {
                            send_response(&mut output, &done, &error_response(&id, -32601)).await?;
                        }
                        (None, None) => {}
                        (None, Some(_)) => return Err(Error::Protocol),
                    }
                } else {
                    let id = message["id"].as_u64().ok_or(Error::Protocol)?;
                    let result = match (message.get("result"), message.get("error")) {
                        (Some(result), None) => Ok(result.clone()),
                        (None, Some(error)) => Err(error["code"]
                            .as_i64()
                            .and_then(|code| i32::try_from(code).ok())
                            .ok_or(Error::Protocol)?),
                        _ => return Err(Error::Protocol),
                    };
                    let reply = pending.remove(&id).ok_or(Error::Protocol)?;
                    let _ = reply.send(result);
                }
            }
            answer = answers.next(), if !answers.is_empty() => {
                let Some((id, epoch, answer)) = answer else { return Err(Error::Protocol) };
                if !incoming.get(&id).is_some_and(|entry| entry.generation == epoch) {
                    continue;
                }
                incoming.remove(&id);
                let response = match answer {
                    Ok(Ok(Ok(result))) => json!({"id": id, "result": result}),
                    Ok(Ok(Err(code))) => error_response(&id, code),
                    Ok(Err(_)) => error_response(&id, -32603),
                    Err(_) => continue,
                };
                send_response(&mut output, &done, &response).await?;
            }
        }
    }
}

pub(super) fn request_id(value: &Value) -> Result<RequestId, Error> {
    let id = serde_json::from_value(value.clone()).map_err(|_| Error::Protocol)?;
    if matches!(&id, RequestId::String(s) if !super::valid_key(s)) {
        return Err(Error::Protocol);
    }
    Ok(id)
}

fn error_response(id: &RequestId, code: i32) -> Value {
    json!({"id": id, "error": {"code": code, "message": "Request not handled"}})
}

async fn send_response(
    output: &mut FramedWrite<impl AsyncWrite + Unpin, LinesCodec>,
    done: &CancellationToken,
    response: &Value,
) -> Result<(), Error> {
    let line = Zeroizing::new(serde_json::to_string(response).map_err(|_| Error::Protocol)?);
    if line.len() > super::super::MAX_LINE {
        return Err(Error::Protocol);
    }
    tokio::select! {
        () = done.cancelled() => Ok(()),
        result = output.send(line.as_str()) => result.map_err(|_| Error::Protocol),
    }
}
