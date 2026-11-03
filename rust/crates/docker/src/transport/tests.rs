use super::*;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::oneshot,
};

type TestError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::test]
async fn failed_driver_does_not_abandon_later_work_even_when_drain_wait_is_cancelled()
-> Result<(), TestError> {
    let transport = Transport::new(Selection {
        api_version: None,
        endpoint: Endpoint::Unix("/unused-transport-regression.sock".into()),
        tls: None,
    })
    .await?;
    let (release, released) = oneshot::channel();
    let (finished, completed) = oneshot::channel();
    {
        let mut owned = transport.0.connections.lock().await;
        let failed = owned.tasks.spawn(std::future::pending());
        failed.abort();
        owned.tasks.spawn(async move {
            released.await.map_err(|_| Error::Transport)?;
            let _ = finished.send(());
            Ok(())
        });
    }
    let mut draining = Box::pin(transport.drain());
    std::future::poll_fn(|context| {
        assert!(draining.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    drop(draining);
    release
        .send(())
        .map_err(|_| "Cancelled drain abandoned later work")?;
    let failure = transport
        .drain()
        .await
        .err()
        .ok_or("Missing driver failure")?;
    assert!(matches!(failure, Error::TransportTask(_)));
    completed.await?;
    assert!(transport.drain().await.is_ok());
    Ok(())
}

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
        let request = String::from_utf8(request)?;
        assert!(request.starts_with("GET /_ping HTTP/1.1\r\n"));
        assert!(
            request
                .to_ascii_lowercase()
                .contains(&format!("\r\nhost: 127.0.0.1:{port}\r\n"))
        );
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
