//! Local App Server JSONL: independent request ownership and ordered ingress.
use crate::Error;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
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

impl Reply {
    pub(super) async fn receive(self) -> Result<Value, CallFailure> {
        self.0
            .await
            .map_err(|_| CallFailure::Uncertain)?
            .map_err(CallFailure::Rejected)
    }
}

impl Client {
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
pub(super) fn connect(
    read: impl AsyncRead + Unpin,
    write: impl AsyncWrite + Unpin,
    done: CancellationToken,
    ingress: impl FnMut(Value) -> Result<Option<Value>, Error>,
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
    mut ingress: impl FnMut(Value) -> Result<Option<Value>, Error>,
) -> Result<(), Error> {
    let codec = || LinesCodec::new_with_max_length(super::super::MAX_LINE);
    let mut input = FramedRead::new(read, codec());
    let mut output = FramedWrite::new(write, codec());
    let mut pending = BTreeMap::new();
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
                    // A server request may reuse the client ID space; it can
                    // never satisfy an outbound request.
                    if let Some(response) = ingress(message)? {
                        let response = serde_json::to_string(&response).map_err(|_| Error::Protocol)?;
                        if response.len() > super::super::MAX_LINE {
                            return Err(Error::Protocol);
                        }
                        tokio::select! {
                            () = done.cancelled() => return Ok(()),
                            result = output.send(response.as_str()) => result.map_err(|_| Error::Protocol)?,
                        }
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
        }
    }
}
