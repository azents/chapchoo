use super::*;
use std::{future::Future, io};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot,
    task::JoinHandle,
};
type TestError = Box<dyn Error + Send + Sync>;

#[derive(Debug)]
struct Request {
    path: String,
    body: String,
}
enum Reply {
    Json(u16, String),
    Redirect,
    Close,
    Hold,
    OversizedHeader,
    OversizedChunks,
}
struct Fixture {
    origin: Url,
    stop: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<Vec<Request>, TestError>>>,
}
impl Fixture {
    async fn start(replies: Vec<Reply>) -> Result<Self, TestError> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let origin = Url::parse(&format!("http://{}/", listener.local_addr()?))?;
        let (stop, mut stopping) = oneshot::channel();
        let task = tokio::spawn(async move {
            let mut requests = Vec::new();
            for reply in replies {
                let (mut stream, _) = tokio::select! { result=listener.accept()=>result?,_=&mut stopping=>return Ok(requests) };
                let mut bytes = Vec::new();
                let headers = loop {
                    if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                        break end + 4;
                    }
                    if bytes.len() > 16384 {
                        return Err("Fixture request headers exceeded bounds".into());
                    }
                    let mut part = [0; 1024];
                    let read = stream.read(&mut part).await?;
                    if read == 0 {
                        return Err("Fixture request ended before headers".into());
                    }
                    bytes.extend_from_slice(&part[..read]);
                };
                let header = std::str::from_utf8(&bytes[..headers])?;
                let path = header
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .ok_or("Missing fixture request path")?
                    .to_owned();
                let length = header
                    .lines()
                    .find_map(|line| {
                        line.split_once(':')
                            .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                            .map(|(_, value)| value.trim().parse::<usize>())
                    })
                    .transpose()?
                    .ok_or("Missing request body length")?;
                if length > 16384 {
                    return Err("Fixture request body exceeded bounds".into());
                }
                while bytes.len() - headers < length {
                    let mut part = [0; 1024];
                    let read = stream.read(&mut part).await?;
                    if read == 0 {
                        return Err("Fixture request body ended early".into());
                    }
                    bytes.extend_from_slice(&part[..read]);
                }
                requests.push(Request {
                    path,
                    body: String::from_utf8(bytes[headers..headers + length].to_vec())?,
                });
                let response=match reply {
                    Reply::Json(status,body)=>format!("HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).into_bytes(),
                    Reply::Redirect=>b"HTTP/1.1 302 Found\r\nLocation: /capture\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
                    Reply::Close=>continue,
                    Reply::Hold=>{let _=stopping.await;return Ok(requests);},
                    Reply::OversizedHeader=>format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",MAX_BODY+1).into_bytes(),
                    Reply::OversizedChunks=>{
                        let mut data=b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".to_vec();
                        for _ in 0..3 { data.extend_from_slice(b"10000\r\n");data.extend(std::iter::repeat_n(b'x',65536));data.extend_from_slice(b"\r\n"); }
                        data.extend_from_slice(b"0\r\n\r\n");data
                    },
                };
                if let Err(error) = stream.write_all(&response).await
                    && !matches!(
                        error.kind(),
                        io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset
                    )
                {
                    return Err(Box::new(error) as TestError);
                }
            }
            Ok(requests)
        });
        Ok(Self {
            origin,
            stop: Some(stop),
            task: Some(task),
        })
    }
    async fn finish(mut self) -> Result<Vec<Request>, TestError> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        self.task.take().ok_or("Missing fixture owner")?.await?
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}
#[derive(Debug)]
struct Failures {
    primary: TestError,
    cleanup: TestError,
}
impl fmt::Display for Failures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Fixture operation failed: {}; cleanup failed: {}",
            self.primary, self.cleanup
        )
    }
}
impl Error for Failures {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.primary.as_ref())
    }
}
async fn exercise<F, Fut>(
    replies: Vec<Reply>,
    timeout: Option<Duration>,
    test: F,
) -> Result<Vec<Request>, TestError>
where
    F: FnOnce(OpenAiDeviceClient) -> Fut,
    Fut: Future<Output = Result<(), TestError>>,
{
    let fixture = Fixture::start(replies).await?;
    let client = (|| -> Result<OpenAiDeviceClient, ClientPreparationError> {
        if let Some(timeout) = timeout {
            OpenAiDeviceClient::from_parts(
                builder()?
                    .https_only(false)
                    .no_proxy()
                    .timeout(timeout)
                    .build()
                    .map_err(preparation)?,
                &fixture.origin,
            )
        } else {
            OpenAiDeviceClient::fixture(&fixture.origin)
        }
    })();
    let primary = match client {
        Ok(client) => test(client).await,
        Err(error) => Err(Box::new(error) as TestError),
    };
    let cleanup = fixture.finish().await;
    match (primary, cleanup) {
        (Ok(()), Ok(requests)) => Ok(requests),
        (Err(primary), Err(cleanup)) => Err(Box::new(Failures { primary, cleanup })),
        (Err(error), _) | (_, Err(error)) => Err(error),
    }
}
fn approval() -> Result<DeviceApproval, TestError> {
    Ok(DeviceApproval {
        authorization_code: ExternalSecret::new(b"code-value".to_vec())?,
        code_verifier: ExternalSecret::new(b"verifier-value".to_vec())?,
    })
}
fn id_token(claims: &str) -> String {
    format!("header.{}.signature", URL_SAFE_NO_PAD.encode(claims))
}

#[tokio::test]
async fn acquisition_maps_real_protocol_shapes_without_model_calls_or_secret_readback()
-> Result<(), TestError> {
    let jwt = id_token(
        r#"{"sub":"not-the-account","email":"synthetic@example.invalid","https://api.openai.com/auth":{"chatgpt_account_id":"account-a","chatgpt_plan_type":"synthetic-plan"}}"#,
    );
    let response=serde_json::json!({"access_token":"synthetic-access","refresh_token":"synthetic-refresh","id_token":jwt,"expires_in":3600}).to_string();
    let requests=exercise(vec![Reply::Json(200,r#"{"device_auth_id":"device-id","usercode":"USER-CODE","interval":"7"}"#.to_owned()),Reply::Json(200,r#"{"authorization_code":"code-value","code_verifier":"verifier-value","code_challenge":"challenge"}"#.to_owned()),Reply::Json(200,response)],None,|client|async move {
        let issued=client.issue(1000).await?;assert_eq!(issued.interval_ms,7000);assert_eq!(issued.expires_at,901000);
        let DevicePollOutcome::Approved(approval)=client.poll(&issued.device_auth_id,&issued.user_code).await? else { return Err("Expected approval".into()); };
        let tokens=client.exchange(&approval,2000).await?;
        assert_eq!(tokens.account_id.as_deref(),Some("account-a"));assert_eq!(tokens.plan_type.as_deref(),Some("synthetic-plan"));assert_eq!(tokens.email.as_deref(),Some("synthetic@example.invalid"));assert_eq!(tokens.expires_at,Some(3_602_000));
        let stored:TokenResponse=serde_json::from_slice(tokens.secret.expose())?;assert_eq!(stored.access_token,"synthetic-access");assert_eq!(stored.refresh_token,"synthetic-refresh");
        assert_eq!(format!("{tokens:?}"),"DeviceAuthenticationTokens([REDACTED])");Ok(())
    }).await?;
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].path, "/api/accounts/deviceauth/usercode");
    assert_eq!(requests[1].path, "/api/accounts/deviceauth/token");
    assert_eq!(requests[2].path, "/oauth/token");
    assert!(requests[2].body.contains("grant_type=authorization_code"));
    assert!(requests[2].body.contains("code=code-value"));
    assert!(requests[2].body.contains("code_verifier=verifier-value"));
    assert!(
        requests[2]
            .body
            .contains("redirect_uri=https%3A%2F%2Fauth.openai.com%2Fdeviceauth%2Fcallback")
    );
    assert!(requests[2].body.contains(CLIENT_ID));
    assert_eq!(
        DEVICE_VERIFICATION_URI,
        "https://auth.openai.com/codex/device"
    );
    Ok(())
}
#[tokio::test]
async fn polling_advances_once_and_classifies_pending_rejection_and_unavailability()
-> Result<(), TestError> {
    let requests = exercise(
        vec![
            Reply::Json(403, "ignored-secret-body".to_owned()),
            Reply::Json(404, String::new()),
            Reply::Json(400, "secret-error-body".to_owned()),
            Reply::Json(429, String::new()),
            Reply::Json(503, String::new()),
            Reply::Redirect,
        ],
        None,
        |client| async move {
            let device = ExternalSecret::new(b"device-id".to_vec())?;
            assert!(matches!(
                client.poll(&device, "CODE").await?,
                DevicePollOutcome::Pending
            ));
            assert!(matches!(
                client.poll(&device, "CODE").await?,
                DevicePollOutcome::Pending
            ));
            assert!(matches!(
                client.issue(1000).await,
                Err(DeviceProviderFailure::Rejected)
            ));
            assert!(matches!(
                client.issue(1000).await,
                Err(DeviceProviderFailure::Unavailable)
            ));
            assert!(matches!(
                client.issue(1000).await,
                Err(DeviceProviderFailure::Unavailable)
            ));
            assert!(matches!(
                client.issue(1000).await,
                Err(DeviceProviderFailure::Rejected)
            ));
            Ok(())
        },
    )
    .await?;
    assert_eq!(requests.len(), 6);
    Ok(())
}
#[tokio::test]
async fn malformed_and_oversized_success_bodies_are_bounded_and_redacted() -> Result<(), TestError>
{
    let requests = exercise(
        vec![
            Reply::Json(200, "synthetic-secret-invalid-json".to_owned()),
            Reply::OversizedHeader,
            Reply::OversizedChunks,
            Reply::Json(
                200,
                r#"{"device_auth_id":"id","user_code":"CODE","interval":0}"#.to_owned(),
            ),
        ],
        None,
        |client| async move {
            for _ in 0..4 {
                let result = client.issue(1000).await;
                assert!(matches!(
                    result,
                    Err(DeviceProviderFailure::InvalidResponse)
                ));
            }
            Ok(())
        },
    )
    .await?;
    assert_eq!(requests.len(), 4);
    Ok(())
}
#[tokio::test]
async fn exchange_lost_response_is_uncertain_and_never_automatically_replayed()
-> Result<(), TestError> {
    let requests = exercise(vec![Reply::Close], None, |client| async move {
        assert!(matches!(
            client.exchange(&approval()?, 1000).await,
            Err(DeviceProviderFailure::Uncertain)
        ));
        Ok(())
    })
    .await?;
    assert_eq!(requests.len(), 1);
    Ok(())
}
#[tokio::test]
async fn stalled_exchange_has_bounded_lifetime_without_authorizing_replay() -> Result<(), TestError>
{
    let requests = exercise(
        vec![Reply::Hold],
        Some(Duration::from_millis(500)),
        |client| async move {
            assert!(matches!(
                client.exchange(&approval()?, 1000).await,
                Err(DeviceProviderFailure::Uncertain)
            ));
            Ok(())
        },
    )
    .await?;
    assert_eq!(requests.len(), 1);
    Ok(())
}
#[tokio::test]
async fn token_expiry_must_remain_representable_by_the_public_protocol() -> Result<(), TestError> {
    let replies = [253_402_300_799_u64, 253_402_300_800_u64]
        .into_iter()
        .map(|seconds| {
            Reply::Json(
                200,
                serde_json::json!({
                    "access_token": "access", "refresh_token": "refresh", "expires_in": seconds,
                })
                .to_string(),
            )
        })
        .collect();
    let requests = exercise(replies, None, |client| async move {
        assert_eq!(
            client.exchange(&approval()?, 999).await?.expires_at,
            Some(MAX_AUTHENTICATION_EXPIRY_MILLIS)
        );
        assert!(matches!(
            client.exchange(&approval()?, 999).await,
            Err(DeviceProviderFailure::InvalidResponse)
        ));
        Ok(())
    })
    .await?;
    assert_eq!(requests.len(), 2);
    Ok(())
}

#[tokio::test]
async fn absent_or_malformed_claims_do_not_fabricate_account_identity_or_expiry()
-> Result<(), TestError> {
    let wrong = id_token(r#"{"sub":"wrong-sub","https://api.openai.com/auth":"wrong-account"}"#);
    let sub = id_token(r#"{"sub":"wrong-sub","plan_type":"wrong-plan"}"#);
    let replies=[Some(wrong),Some(sub),None].into_iter().map(|id_token|Reply::Json(200,serde_json::json!({"access_token":"access","refresh_token":"refresh","id_token":id_token}).to_string())).collect();
    let requests = exercise(replies, None, |client| async move {
        for _ in 0..3 {
            let tokens = client.exchange(&approval()?, 1000).await?;
            assert!(tokens.account_id.is_none());
            assert!(tokens.plan_type.is_none());
            assert!(tokens.expires_at.is_none());
        }
        Ok(())
    })
    .await?;
    assert_eq!(requests.len(), 3);
    Ok(())
}
