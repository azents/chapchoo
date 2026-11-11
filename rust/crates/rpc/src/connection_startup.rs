//! Client-owned startup dispatch. It never runs inside Connector stream admission.
use crate::PeerDelivery;
use chapchoo_domain::HarnessSupply;
use chapchoo_protocol::{control, conversations as wire};
use chapchoo_service::{
    ComputersService, ConnectionStartupAdmission, ConnectionStartupContext, ConversationService,
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::watch,
    task::{Id, JoinError, JoinSet},
    time::{Instant, MissedTickBehavior},
};

const PAGE_LIMIT: u32 = 20;
const PAGE: usize = PAGE_LIMIT as usize;
const DISPATCH_BACKOFF: Duration = Duration::from_secs(15);

pub async fn reconcile_connection_startups(
    computers: Arc<ComputersService>,
    conversation: Arc<ConversationService>,
    delivery: PeerDelivery,
    admission: Arc<AtomicBool>,
    mut stop: watch::Receiver<bool>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut after = None;
    let mut next_attempt = HashMap::<String, Instant>::new();
    let mut inflight = HashMap::<Id, String>::new();
    let mut tasks = JoinSet::new();
    loop {
        if stopped(&stop) {
            break;
        }
        tokio::select! {
            biased;
            changed = stop.changed() => {
                if changed.is_err() || *stop.borrow() {
                    break;
                }
            }
            _ = interval.tick() => {
                if !admission.load(Ordering::Acquire) || stopped(&stop) || inflight.len() >= PAGE {
                    continue;
                }
                let page = match computers.running_computer_ids(after.as_deref(), PAGE_LIMIT).await {
                    Ok(page) => page,
                    Err(error) => {
                        tracing::warn!(error = ?error, "Connection startup page failed");
                        continue;
                    }
                };
                let wrapped = page.len() < PAGE;
                after = if wrapped { None } else { page.last().cloned() };
                let now = Instant::now();
                next_attempt.retain(|_, until| *until > now);
                for computer_id in page {
                    if stopped(&stop) || inflight.len() >= PAGE {
                        break;
                    }
                    if next_attempt.get(&computer_id).is_some_and(|until| *until > now)
                        || inflight.values().any(|existing| existing == &computer_id)
                    {
                        continue;
                    }
                    let computers = computers.clone();
                    let conversation = conversation.clone();
                    let delivery = delivery.clone();
                    let task_stop = stop.clone();
                    let id = computer_id.clone();
                    let handle = tasks.spawn(async move {
                        dispatch_computer(&computers, &conversation, &delivery, &task_stop, &id).await
                    });
                    inflight.insert(handle.id(), computer_id);
                }
            }
            finished = tasks.join_next_with_id(), if !tasks.is_empty() => {
                record_finish(&mut inflight, &mut next_attempt, finished);
            }
        }
    }
    while let Some(finished) = tasks.join_next_with_id().await {
        record_finish(&mut inflight, &mut next_attempt, Some(finished));
    }
}

fn stopped(stop: &watch::Receiver<bool>) -> bool {
    *stop.borrow() || stop.has_changed().is_err()
}

fn record_finish(
    inflight: &mut HashMap<Id, String>,
    next_attempt: &mut HashMap<String, Instant>,
    finished: Option<Result<(Id, Pace), JoinError>>,
) {
    let Some(finished) = finished else {
        return;
    };
    match finished {
        Ok((id, pace)) => {
            if let Some(computer_id) = inflight.remove(&id)
                && pace == Pace::Later
            {
                next_attempt.insert(computer_id, Instant::now() + DISPATCH_BACKOFF);
            }
        }
        Err(error) => {
            inflight.remove(&error.id());
            tracing::warn!(error = ?error, "Connection startup task failed");
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pace {
    Soon,
    Later,
}

async fn dispatch_computer(
    computers: &ComputersService,
    conversation: &ConversationService,
    delivery: &PeerDelivery,
    stop: &watch::Receiver<bool>,
    computer_id: &str,
) -> Pace {
    if stopped(stop) {
        return Pace::Soon;
    }
    let route = match computers.resolve_connector_route(computer_id).await {
        Ok(Some(route)) => route,
        Ok(None) => return Pace::Soon,
        Err(error) => {
            tracing::warn!(error = ?error, "Connection startup route lookup failed");
            return Pace::Soon;
        }
    };
    if stopped(stop) {
        return Pace::Soon;
    }
    let admissions = match conversation
        .admit_connection_startup(computer_id, &route)
        .await
    {
        Ok(admissions) => admissions,
        Err(error) => {
            tracing::warn!(error = ?error, "Connection startup admission failed");
            return Pace::Soon;
        }
    };
    if admissions.is_empty() {
        return Pace::Soon;
    }
    let mut attempted = false;
    for admission in admissions {
        if stopped(stop) {
            break;
        }
        let ConnectionStartupAdmission::Dispatch {
            agent_id,
            conversation_id,
            operation_id,
            computer_request_id,
            execution_id,
            requested_supply,
            fresh_native_allowed,
            require_retained_source,
            context,
        } = admission
        else {
            continue;
        };
        let allowed = match conversation
            .connection_startup_may_dispatch(computer_id, &route, &operation_id)
            .await
        {
            Ok(allowed) => allowed,
            Err(error) => {
                tracing::warn!(error = ?error, "Connection startup dispatch gate failed");
                continue;
            }
        };
        if !allowed || stopped(stop) {
            continue;
        }
        let Some(call_id) = call_id() else {
            tracing::warn!("Connection startup call identity generation failed");
            continue;
        };
        attempted = true;
        let setup_context = setup_context(context, fresh_native_allowed, require_retained_source);
        let computer = control::ComputerContext {
            computer_id: computer_id.to_owned(),
            request_id: computer_request_id.clone(),
            execution_id,
        };
        let invocation = control::ConnectorInvocation {
            computer_id: computer_id.to_owned(),
            owner: None,
            authority: Some(control::PeerAuthority {
                context: Some(control::peer_authority::Context::Reconciliation(
                    control::ReconciliationAuthority {
                        computer: Some(computer.clone()),
                        owner: Some(control::ExpectedOwner {
                            instance_id: route.component.control_id.clone(),
                            connection_id: route.component.connection_id.clone(),
                        }),
                    },
                )),
            }),
            command: Some(control::ConnectorCommand {
                identity: Some(control::CallIdentity {
                    request_id: computer_request_id,
                    call_id,
                    connection_id: String::new(),
                }),
                computer: Some(computer),
                remaining_ms: 10_000,
                operation: Some(control::connector_command::Operation::Setup(
                    wire::SetupCommand {
                        agent_id,
                        conversation_id,
                        operation_id,
                        supply: Some(wire_supply(requested_supply)),
                        context: Some(setup_context),
                    },
                )),
            }),
        };
        // Delivery owns this call until the peer attempt settles. Stopping the
        // reconciler must not drop it and reconstruct permission from silence.
        if let Err(error) = delivery
            .connector(invocation, Instant::now() + Duration::from_secs(10))
            .await
        {
            tracing::warn!(error = ?error, "Connection startup dispatch failed");
        }
    }
    if attempted { Pace::Later } else { Pace::Soon }
}

fn call_id() -> Option<String> {
    chapchoo_crypto::BearerSecret::generate()
        .ok()
        .map(|secret| {
            secret
                .digest()
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect()
        })
}

fn wire_supply(supply: HarnessSupply) -> wire::HarnessSupply {
    wire::HarnessSupply {
        choice: Some(match supply {
            HarnessSupply::Managed => wire::harness_supply::Choice::Managed(()),
            HarnessSupply::Existing(path) => {
                wire::harness_supply::Choice::ExistingCodexPath(path.as_str().into())
            }
        }),
    }
}

fn setup_context(
    context: ConnectionStartupContext,
    fresh: bool,
    require_retained_source: bool,
) -> wire::setup_command::Context {
    match context {
        ConnectionStartupContext::Initial => {
            wire::setup_command::Context::Initial(wire::InitialNativeContext {
                allow_fresh: fresh,
                require_retained_source,
            })
        }
        ConnectionStartupContext::Existing {
            snapshot,
            native_session,
        } => wire::setup_command::Context::Existing(wire::NativeTarget {
            snapshot_id: snapshot.as_str().into(),
            native_session_id: native_session.as_str().into(),
        }),
    }
}
