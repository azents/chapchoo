//! Bounded local IO for the same generated clients, codecs and handlers as TCP.
use hyper_util::rt::TokioIo;
use std::{io, time::Duration};
use tokio::{io::DuplexStream, sync::mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::{Channel, Endpoint};
use tower::service_fn;

pub type MemoryIncoming = ReceiverStream<Result<DuplexStream, io::Error>>;

/// The composition root owns the incoming server task and its shutdown. No peer
/// socket, direct handler shortcut, or detached server lifecycle is introduced.
pub fn memory_transport() -> (Channel, MemoryIncoming) {
    let (sender, receiver) = mpsc::channel(4);
    let connector = service_fn(move |_| {
        let sender = sender.clone();
        async move {
            let (client, server) = tokio::io::duplex(64 * 1024);
            sender.try_send(Ok(server)).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::ConnectionRefused,
                    "Memory listener is unavailable",
                )
            })?;
            Ok::<_, io::Error>(TokioIo::new(client))
        }
    });
    let channel = Endpoint::from_static("http://chapchoo.memory")
        .buffer_size(32)
        .concurrency_limit(32)
        .timeout(Duration::from_secs(10))
        .connect_with_connector_lazy(connector);
    (channel, ReceiverStream::new(receiver))
}
