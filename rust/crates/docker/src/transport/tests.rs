use super::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::oneshot,
};

type TestError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::test]
async fn selected_connection_and_drain_outlive_headers_until_sdk_body_completion()
-> Result<(), TestError> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let (headers, headers_received) = oneshot::channel();
    let (finish, body_allowed) = oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await?;
        let mut request = Vec::new();
        let mut byte = [0];
        while !request.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).await?;
            request.extend_from_slice(&byte);
            if request.len() > 16 * 1024 {
                return Err::<(), TestError>("Unbounded fixture request".into());
            }
        }
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n")
            .await?;
        let _ = headers.send(());
        body_allowed.await?;
        stream.write_all(b"OK").await?;
        stream.shutdown().await?;
        Ok(())
    });
    let transport = Transport::new(Selection {
        api_version: None,
        endpoint: Endpoint::Tcp {
            host: "127.0.0.1".into(),
            port,
        },
        tls: None,
    })
    .await?;
    let sdk = transport.sdk()?;
    let request = tokio::spawn(async move { sdk.ping().await });
    headers_received.await?;
    assert_eq!(transport.0.permits.available_permits(), CONNECTIONS - 1);
    let draining = transport.drain();
    tokio::pin!(draining);
    // Poll the actual drain first; it must remain pending behind body completion.
    tokio::select! {
        biased;
        _ = &mut draining => return Err("Transport drained at headers".into()),
        _ = std::future::ready(()) => {}
    }
    assert!(!transport.0.admitting.load(Ordering::Acquire));
    finish.send(()).map_err(|_| "Fixture body receiver lost")?;
    request.await??;
    draining.await?;
    server.await??;
    assert_eq!(transport.0.permits.available_permits(), CONNECTIONS);
    assert!(transport.sdk()?.ping().await.is_err());
    Ok(())
}
