//! Exact, execution-local App Server ingress and operations.
use super::{
    Thread, TurnOptions, call, events, forms, initialize, model_catalog, service_tier, tools, wire,
};
use crate::{CaptureLog, NativeMcpDeclaration, ProtocolStage, runtime::*};
use serde_json::{Value, json};
use std::{
    future::Future,
    path::Path,
    sync::{Mutex as SyncMutex, OnceLock},
};
use tokio::io::{AsyncRead, AsyncWrite};

struct Ingress {
    capture: CaptureLog,
    observations: Arc<Observations>,
    root: OnceLock<Root>,
    requests: SyncMutex<BTreeMap<String, LiveRequest>>,
}

struct LiveRequest {
    turn: String,
    position: chapchoo_domain::SourcePosition,
    kind: RequestKind,
}

enum RequestKind {
    Permission(tools::Approval),
    Form(forms::Form),
}

impl LiveRequest {
    fn current(&self, turn: Option<&str>) -> bool {
        Some(self.turn.as_str()) == turn
            && match &self.kind {
                RequestKind::Permission(approval) => !approval.cancelled(),
                RequestKind::Form(form) => !form.cancelled(),
            }
    }
}

struct Root {
    target: NativeTextTarget,
    insertions: events::Insertions,
}

pub(in crate::runtime) struct Connection {
    client: wire::Client,
    ingress: Arc<Ingress>,
}

pub(in crate::runtime) struct LiveThread {
    connection: Connection,
    id: NativeConversationId,
    options: NativeModelOptions,
}

impl Connection {
    pub(in crate::runtime) fn connect(
        read: impl AsyncRead + Unpin,
        write: impl AsyncWrite + Unpin,
        done: CancellationToken,
        capture: CaptureLog,
        observations: Arc<Observations>,
    ) -> (Self, impl Future<Output = Result<(), Error>>) {
        let ingress = Arc::new(Ingress {
            capture,
            observations,
            root: OnceLock::new(),
            requests: SyncMutex::default(),
        });
        let receive = ingress.clone();
        let (client, driver) = wire::connect_with_requests(read, write, done, move |raw, frame| {
            receive.record(raw, frame)
        });
        (Self { client, ingress }, driver)
    }

    pub(in crate::runtime) async fn open(
        self,
        cwd: &Path,
        context: &NativeContext,
        mcp: &NativeMcpDeclaration,
        initial_key: Option<&str>,
        empty_scope: bool,
    ) -> Result<LiveThread, Error> {
        let existing = match context {
            NativeContext::New => None,
            NativeContext::Resume {
                session,
                replay: CaptureReplay::AlreadyCaptured,
            } => Some(session),
            NativeContext::Resume {
                replay: CaptureReplay::FromStart,
                ..
            } => return Err(Error::CaptureConflict),
        };
        initialize(&self.client, initial_key).await?;
        let opened = Thread::open(&self.client, cwd, existing, Some(mcp)).await?;
        let id = opened.id;
        if existing.is_none() {
            call(
                &self.client,
                ProtocolStage::Session,
                "thread/name/set",
                json!({
                    "threadId": id.as_str(), "name": "Agent project",
                }),
            )
            .await?;
        }
        let options = tokio::time::timeout(Duration::from_secs(5), model_catalog(&self.client))
            .await
            .ok()
            .and_then(Result::ok)
            .unwrap_or_default()
            .into_iter()
            .map(|candidate| NativeModelChoice {
                capabilities: Some(candidate.capabilities()),
                id: candidate.id,
                label: candidate.label,
            })
            .collect::<Vec<_>>();
        let current = options
            .iter()
            .find(|choice| Some(&choice.id) == opened.model.as_ref())
            .cloned();
        call(
            &self.client,
            ProtocolStage::ServerTools,
            "mcpServerStatus/list",
            json!({
                "threadId": id.as_str(), "serverName": "chapchoo",
            }),
        )
        .await?;
        self.ingress.capture.bind_native(&id)?;
        let snapshot = self.ingress.capture.snapshot()?;
        self.ingress
            .observations
            .bind(snapshot.clone(), id.clone(), empty_scope)?;
        self.ingress
            .root
            .set(Root {
                target: NativeTextTarget {
                    snapshot,
                    native_session: id.clone(),
                },
                insertions: events::Insertions::new(id.clone()),
            })
            .map_err(|_| Error::CaptureConflict)?;
        Ok(LiveThread {
            connection: self,
            id,
            options: NativeModelOptions {
                option_id: "model".into(),
                current,
                options,
            },
        })
    }
}

impl Ingress {
    fn record(&self, raw: &str, frame: Value) -> Result<Option<wire::ServerReply>, Error> {
        let Some(root) = self.root.get() else {
            return Ok(None);
        };
        if frame["params"]["threadId"].as_str() != Some(root.target.native_session.as_str()) {
            return Ok(None);
        }
        let method = frame["method"].as_str().ok_or(Error::Protocol)?;
        if method == "mcpServer/elicitation/request" && frame.get("id").is_some() {
            return self.observations.with_work(|turn| {
                let Some(turn) = turn else { return Ok(None) };
                let approval = tools::Approval::decode(&root.target, turn, &frame);
                let form = if approval.is_none() {
                    forms::Form::offer(
                        root.target.native_session.as_str(),
                        turn,
                        frame["params"].clone(),
                    )
                } else {
                    None
                };
                if approval.is_none() && form.is_none() {
                    return Ok(None);
                }
                let mut pending = self.requests.lock().map_err(|_| Error::Protocol)?;
                pending.retain(|_, request| request.current(Some(turn)));
                let position = self.capture.append(raw, None)?;
                let (kind, reply) = if let Some(details) = approval {
                    let (approval, reply) = tools::Approval::prepare(
                        root.target.clone(),
                        position.clone(),
                        turn,
                        details,
                    );
                    (RequestKind::Permission(approval), reply)
                } else {
                    let (form, reply) = form.ok_or(Error::Protocol)?;
                    (RequestKind::Form(form), reply)
                };
                pending.insert(
                    observations::token("request")?,
                    LiveRequest {
                        turn: turn.to_owned(),
                        position,
                        kind,
                    },
                );
                self.observations.changed();
                Ok(Some(reply))
            });
        }
        if frame.get("id").is_some() {
            return Ok(None);
        }
        match method {
            "turn/started" | "turn/completed" => {
                self.observations
                    .record_app_server_turn(&self.capture, raw, &root.target)?
            }
            "item/started" | "item/completed" | "item/agentMessage/delta" => {
                if matches!(method, "item/started" | "item/completed")
                    && let Some(turn) = frame["params"]["turnId"].as_str()
                {
                    self.observations.record_app_server_item(
                        &root.target.native_session,
                        turn,
                        &frame["params"]["item"],
                        method == "item/completed",
                    )?;
                }
                self.observations.with_work(|turn| {
                    if turn.is_some() && frame["params"]["turnId"].as_str() == turn {
                        if method == "item/started" {
                            root.insertions.observe(&frame)?;
                        } else {
                            self.capture.append(raw, None)?;
                        }
                    }
                    Ok(())
                })?;
            }
            _ => {}
        }
        Ok(None)
    }
}

impl LiveThread {
    pub(in crate::runtime) fn id(&self) -> &str {
        self.id.as_str()
    }
    pub(in crate::runtime) fn is_closed(&self) -> bool {
        self.connection.client.is_closed()
    }

    fn thread(&self) -> Thread<'_> {
        Thread {
            link: &self.connection.client,
            id: self.id.clone(),
            model: None,
        }
    }

    pub(in crate::runtime) fn models(&self) -> Option<NativeModelOptions> {
        (!self.options.options.is_empty()).then(|| self.options.clone())
    }

    pub(in crate::runtime) fn enqueue_prompt(
        &self,
        text: &ConversationText,
        model: Option<&str>,
        tuning: &chapchoo_domain::ConversationModelTuning,
    ) -> Result<impl Future<Output = Result<String, Error>> + use<>, Error> {
        if model.is_some_and(|id| {
            !self
                .options
                .options
                .iter()
                .any(|candidate| candidate.id == id)
        }) {
            return Err(Error::NotAdmitted);
        }
        if !tuning.is_inherited()
            && !self.options.options.iter().any(|candidate| {
                Some(candidate.id.as_str()) == model
                    && candidate
                        .capabilities
                        .as_ref()
                        .is_some_and(|options| options.accepts(tuning))
            })
        {
            return Err(Error::NotAdmitted);
        }
        let root = self
            .connection
            .ingress
            .root
            .get()
            .ok_or(Error::NotAdmitted)?;
        let pending = self.thread().submit_with_options(
            &root.insertions,
            text,
            TurnOptions {
                model,
                effort: tuning.effort.as_deref(),
                tier: tuning.speed.map(service_tier),
            },
        )?;
        Ok(async move { Ok(pending.await?.item) })
    }

    pub(in crate::runtime) fn steering(
        &self,
        text: &ConversationText,
    ) -> Result<Option<impl Future<Output = Result<String, Error>> + use<>>, Error> {
        let ingress = &self.connection.ingress;
        let root = ingress.root.get().ok_or(Error::NotAdmitted)?;
        ingress.observations.with_work(|turn| {
            turn.map(|turn| {
                let pending = self.thread().steer(&root.insertions, turn, text)?;
                Ok(async move { Ok(pending.await?.item) })
            })
            .transpose()
        })
    }

    pub(in crate::runtime) fn stop(&self, turn: &str) -> Result<(), Error> {
        let _receipt = self.thread().interrupt(turn)?;
        Ok(())
    }

    pub(in crate::runtime) async fn close(&self) {
        let ingress = &self.connection.ingress;
        if let Ok(Some(receipt)) = ingress
            .observations
            .with_work(|turn| turn.map(|turn| self.thread().interrupt(turn)).transpose())
        {
            let _ = tokio::time::timeout(Duration::from_secs(2), receipt.receive()).await;
        }
    }

    pub(in crate::runtime) fn pending(&self) -> Result<Vec<InteractiveRequest>, Error> {
        let ingress = &self.connection.ingress;
        ingress.observations.with_work(|turn| {
            let mut requests = ingress.requests.lock().map_err(|_| Error::Protocol)?;
            requests.retain(|_, request| request.current(turn));
            Ok(requests
                .iter()
                .map(|(id, request)| match &request.kind {
                    RequestKind::Permission(approval) => InteractiveRequest::Permission {
                        id: id.clone(),
                        occurrence: request.position.clone(),
                        request: approval.question(self.id()),
                    },
                    RequestKind::Form(form) => InteractiveRequest::Form {
                        id: id.clone(),
                        occurrence: request.position.clone(),
                        request: form.request.clone(),
                    },
                })
                .collect())
        })
    }

    pub(in crate::runtime) fn respond(
        &self,
        response: InteractiveResponse,
        exact: Option<&chapchoo_domain::SourcePosition>,
    ) -> Result<(), Error> {
        let id = match &response {
            InteractiveResponse::Permission { id, .. } | InteractiveResponse::Form { id, .. } => {
                id.clone()
            }
        };
        let ingress = &self.connection.ingress;
        let root = ingress.root.get().ok_or(Error::NotAdmitted)?;
        ingress.observations.with_work(|turn| {
            let mut requests = ingress.requests.lock().map_err(|_| Error::Protocol)?;
            let request = requests.get_mut(&id).ok_or(Error::NotAdmitted)?;
            if !request.current(turn) || exact.is_some_and(|position| *position != request.position)
            {
                return Err(Error::NotAdmitted);
            }
            match (&mut request.kind, response) {
                (
                    RequestKind::Permission(approval),
                    InteractiveResponse::Permission { response, .. },
                ) => {
                    let choice = match response.outcome {
                        schema::RequestPermissionOutcome::Selected(selected) => {
                            match selected.option_id.0.as_ref() {
                                "approve-once" => tools::Choice::ApproveOnce,
                                "decline" => tools::Choice::Decline,
                                _ => return Err(Error::NotAdmitted),
                            }
                        }
                        schema::RequestPermissionOutcome::Cancelled(_) => tools::Choice::Decline,
                        _ => return Err(Error::NotAdmitted),
                    };
                    approval.answer(
                        &root.target,
                        &request.position,
                        turn.ok_or(Error::NotAdmitted)?,
                        choice,
                    )?;
                    requests.remove(&id);
                }
                (RequestKind::Form(form), InteractiveResponse::Form { response, .. }) => {
                    let reply = form.prepare_reply(response)?;
                    let Some(LiveRequest {
                        kind: RequestKind::Form(form),
                        ..
                    }) = requests.remove(&id)
                    else {
                        return Err(Error::Protocol);
                    };
                    form.answer(reply)?;
                }
                _ => return Err(Error::NotAdmitted),
            }
            Ok(())
        })
    }
}
