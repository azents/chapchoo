use super::*;
use tokio::io::AsyncReadExt;

#[tokio::test]
async fn steering_only_prompts_after_explicit_non_consumption() -> Result<(), TestError> {
    for (response, prompt_expected, outcome) in [
        (json!({"outcome":"injected"}), false, "steered"),
        (json!({"outcome":"failed"}), false, "failed"),
        (json!({"outcome":"startedNewTurn"}), false, "error"),
        (
            json!({"outcome":"promptRequired","reason":"different"}),
            false,
            "error",
        ),
        (json!({"outcome":"promptRequired"}), false, "error"),
        (
            json!({"outcome":"promptRequired","reason":"noRunningTurn"}),
            true,
            "inserted",
        ),
    ] {
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
        reply(&mut write, &initialize, initialization(2)).await?;
        let new = request(&mut read).await?;
        reply(&mut write, &new, json!({"sessionId":"native"})).await?;
        let session = rx.await??;
        let text = ConversationText::parse("exact \n한글".into())?;
        let submitted = steering::submit(&session, &text);
        tokio::pin!(submitted);
        let steer = tokio::select! { request=request(&mut read)=>request?,_=&mut submitted=>return Err("Premature steering result".into()) };
        assert_eq!(steer.method, "_session/steering");
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct Input {
            session_id: String,
            prompt: Vec<schema::ContentBlock>,
            #[serde(rename = "_meta")]
            meta: Meta,
        }
        #[derive(Deserialize)]
        struct Meta {
            steering: Idle,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Idle {
            idle_behavior: String,
        }
        let input: Input = serde_json::from_value(steer.params.clone())?;
        assert_eq!(input.session_id, "native");
        assert_eq!(input.meta.steering.idle_behavior, "promptRequired");
        assert_eq!(
            input.prompt,
            vec![schema::ContentBlock::from(text.as_str())]
        );
        reply(&mut write, &steer, response).await?;
        if prompt_expected {
            let prompt = tokio::select! { request=request(&mut read)=>request?,_=&mut submitted=>return Err("Expected ordinary prompt after explicit non-consumption".into()) };
            assert_eq!(prompt.method, "session/prompt");
            let input: schema::PromptRequest = serde_json::from_value(prompt.params.clone())?;
            assert_eq!(
                input.prompt,
                vec![schema::ContentBlock::from(text.as_str())]
            );
            reply(&mut write, &prompt, json!({"messageId":"exact-insertion"})).await?;
        }
        let actual = submitted.await;
        match (outcome, actual) {
            ("steered", Ok(NativePromptAcceptance::Steered))
            | ("failed", Ok(NativePromptAcceptance::Failed))
            | ("error", Err(Error::Protocol)) => {}
            ("inserted", Ok(NativePromptAcceptance::Inserted { native_message_id })) => {
                assert_eq!(native_message_id, "exact-insertion")
            }
            _ => return Err("Unexpected steering outcome".into()),
        }
        done.cancel();
        let _ = connection.await?;
        let mut trailing = String::new();
        read.read_to_string(&mut trailing).await?;
        for line in trailing.lines() {
            #[derive(Deserialize)]
            struct Method {
                method: Option<String>,
            }
            assert_ne!(
                serde_json::from_str::<Method>(line)?.method.as_deref(),
                Some("session/prompt")
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn cancelling_a_steering_waiter_after_publication_never_grants_prompt_fallback()
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
    reply(&mut write, &initialize, initialization(2)).await?;
    let new = request(&mut read).await?;
    reply(&mut write, &new, json!({"sessionId":"native"})).await?;
    let session = rx.await??;
    let text = ConversationText::parse("once".into())?;
    {
        let submitted = steering::submit(&session, &text);
        tokio::pin!(submitted);
        let steer = tokio::select! { request=request(&mut read)=>request?,_=&mut submitted=>return Err("Premature steering result".into()) };
        assert_eq!(steer.method, "_session/steering");
        // The response is deliberately unavailable after native publication.
    }
    done.cancel();
    connection.await??;
    let mut trailing = String::new();
    read.read_to_string(&mut trailing).await?;
    for line in trailing.lines() {
        #[derive(Deserialize)]
        struct Method {
            method: Option<String>,
        }
        assert_ne!(
            serde_json::from_str::<Method>(line)?.method.as_deref(),
            Some("session/prompt")
        );
    }
    Ok(())
}
