use super::*;
#[path = "request_tests.rs"]
mod request_tests;
#[path = "steering_tests.rs"]
mod steering_tests;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, duplex};
#[path = "authentication_tests.rs"]
mod authentication_tests;

type TestError = Box<dyn std::error::Error + Send + Sync>;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    id: schema::RequestId,
    method: String,
    params: Value,
}
async fn request(
    reader: &mut BufReader<tokio::io::ReadHalf<tokio::io::DuplexStream>>,
) -> Result<Request, TestError> {
    let mut line = String::new();
    let size = reader.read_line(&mut line).await?;
    if size == 0 {
        return Err("Fixture transport closed".into());
    }
    Ok(serde_json::from_str(&line)?)
}
async fn reply(
    writer: &mut tokio::io::WriteHalf<tokio::io::DuplexStream>,
    request: &Request,
    result: Value,
) -> Result<(), TestError> {
    writer
        .write_all(
            format!(
                "{}\n",
                json!({"jsonrpc":"2.0","id":request.id,"result":result})
            )
            .as_bytes(),
        )
        .await?;
    Ok(())
}
fn initialization(version: u32) -> Value {
    json!({"protocolVersion":version,"info":{"name":"fixture","version":"1"},"capabilities":{"session":{}}})
}
#[tokio::test]
async fn existing_capture_resumes_without_requesting_native_history_again() -> Result<(), TestError>
{
    let (sdk, peer) = duplex(65536);
    let (read, write) = tokio::io::split(sdk);
    let outgoing = SinkExt::<String>::sink_map_err(
        FramedWrite::new(write, LinesCodec::new()),
        std::io::Error::other,
    );
    let incoming =
        FramedRead::new(read, LinesCodec::new()).map(|r| r.map_err(std::io::Error::other));
    let (ready, rx) = oneshot::channel();
    let done = CancellationToken::new();
    let connection = tokio::spawn(connect(
        Lines::new(outgoing, incoming),
        Arc::default(),
        Arc::default(),
        ready,
        done.clone(),
        "/workspace".into(),
        NativeContext::Resume {
            session: NativeConversationId::parse("retained".into())?,
            replay: CaptureReplay::AlreadyCaptured,
        },
    ));
    let (read, mut write) = tokio::io::split(peer);
    let mut read = BufReader::new(read);
    let initialize = request(&mut read).await?;
    reply(&mut write, &initialize, initialization(2)).await?;
    let resume = request(&mut read).await?;
    assert_eq!(resume.method, "session/resume");
    let params: schema::ResumeSessionRequest = serde_json::from_value(resume.params.clone())?;
    assert_eq!(params.session_id.to_string(), "retained");
    assert!(params.replay_from.is_none());
    reply(&mut write, &resume, json!({})).await?;
    assert_eq!(rx.await??.session_id().to_string(), "retained");
    done.cancel();
    connection.await??;
    Ok(())
}
#[tokio::test]
async fn prompt_ack_retains_owner_and_does_not_require_completion_or_link() -> Result<(), TestError>
{
    let (sdk, peer) = duplex(65536);
    let (read, write) = tokio::io::split(sdk);
    let outgoing = SinkExt::<String>::sink_map_err(
        FramedWrite::new(write, LinesCodec::new()),
        std::io::Error::other,
    );
    let incoming =
        FramedRead::new(read, LinesCodec::new()).map(|r| r.map_err(std::io::Error::other));
    let (ready, rx) = oneshot::channel();
    let done = CancellationToken::new();
    let connection = tokio::spawn(connect(
        Lines::new(outgoing, incoming),
        Arc::default(),
        Arc::default(),
        ready,
        done.clone(),
        "/workspace".into(),
        NativeContext::New,
    ));
    let (read, mut write) = tokio::io::split(peer);
    let mut read = BufReader::new(read);
    let initialize = request(&mut read).await?;
    assert_eq!(initialize.method, "initialize");
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Initialize {
        protocol_version: u32,
    }
    assert_eq!(
        serde_json::from_value::<Initialize>(initialize.params.clone())?.protocol_version,
        2
    );
    reply(&mut write, &initialize, initialization(2)).await?;
    let new = request(&mut read).await?;
    assert_eq!(new.method, "session/new");
    reply(&mut write, &new, json!({"sessionId":"native-1"})).await?;
    let session = rx.await??;
    let prompt = session.send_prompt("verbatim \ntext").block_task();
    tokio::pin!(prompt);
    // Poll the owned request while the fixture receives its exact publication.
    let prompt_request = tokio::select! {
        r=request(&mut read)=>r?,
        _=&mut prompt=>return Err("Prompt completed before native acknowledgement".into()),
    };
    assert_eq!(prompt_request.method, "session/prompt");
    reply(
        &mut write,
        &prompt_request,
        json!({"messageId":"exact-native-message"}),
    )
    .await?;
    assert_eq!(prompt.await?.message_id.to_string(), "exact-native-message");
    assert!(!connection.is_finished());
    // No running/idle update was emitted: the ACK is deliberately not completion.
    session.cancel_active_work()?;
    let mut line = String::new();
    read.read_line(&mut line).await?;
    #[derive(Deserialize)]
    struct Notification {
        method: String,
    }
    assert_eq!(
        serde_json::from_str::<Notification>(&line)?.method,
        "session/cancel"
    );
    done.cancel();
    connection.await??;
    Ok(())
}
#[tokio::test]
async fn unsupported_v1_does_not_create_a_fallback_session() -> Result<(), TestError> {
    let (sdk, peer) = duplex(65536);
    let (read, write) = tokio::io::split(sdk);
    let outgoing = SinkExt::<String>::sink_map_err(
        FramedWrite::new(write, LinesCodec::new()),
        std::io::Error::other,
    );
    let incoming =
        FramedRead::new(read, LinesCodec::new()).map(|r| r.map_err(std::io::Error::other));
    let (ready, rx) = oneshot::channel();
    let connection = tokio::spawn(connect(
        Lines::new(outgoing, incoming),
        Arc::default(),
        Arc::default(),
        ready,
        CancellationToken::new(),
        "/workspace".into(),
        NativeContext::New,
    ));
    let (read, mut write) = tokio::io::split(peer);
    let mut read = BufReader::new(read);
    let initialize = request(&mut read).await?;
    reply(&mut write, &initialize, initialization(1)).await?;
    assert!(connection.await?.is_err());
    assert!(rx.await?.is_err());
    let mut extra = String::new();
    assert_eq!(read.read_line(&mut extra).await?, 0);
    Ok(())
}
#[tokio::test]
async fn resume_requests_start_replay_and_failure_never_creates_fresh_context()
-> Result<(), TestError> {
    let (sdk, peer) = duplex(65536);
    let (read, write) = tokio::io::split(sdk);
    let outgoing = SinkExt::<String>::sink_map_err(
        FramedWrite::new(write, LinesCodec::new()),
        std::io::Error::other,
    );
    let incoming =
        FramedRead::new(read, LinesCodec::new()).map(|r| r.map_err(std::io::Error::other));
    let (ready, rx) = oneshot::channel();
    let id = NativeConversationId::parse("original".into())?;
    let connection = tokio::spawn(connect(
        Lines::new(outgoing, incoming),
        Arc::default(),
        Arc::default(),
        ready,
        CancellationToken::new(),
        "/workspace".into(),
        NativeContext::Resume {
            session: id,
            replay: CaptureReplay::FromStart,
        },
    ));
    let (read, mut write) = tokio::io::split(peer);
    let mut read = BufReader::new(read);
    let initialize = request(&mut read).await?;
    reply(&mut write, &initialize, initialization(2)).await?;
    let resume = request(&mut read).await?;
    assert_eq!(resume.method, "session/resume");
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Resume {
        session_id: String,
        replay_from: schema::ReplayFrom,
    }
    let target: Resume = serde_json::from_value(resume.params)?;
    assert_eq!(target.session_id, "original");
    assert!(matches!(target.replay_from, schema::ReplayFrom::Start(_)));
    write.write_all(format!("{}\n",json!({"jsonrpc":"2.0","id":resume.id,"error":{"code":-32001,"message":"Context unavailable"}})).as_bytes()).await?;
    assert!(connection.await?.is_err());
    assert!(rx.await?.is_err());
    let mut extra = String::new();
    assert_eq!(read.read_line(&mut extra).await?, 0);
    Ok(())
}
