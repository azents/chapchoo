//! Direct, reusable peer channels with one explicit pre-forwarding reroute.
use crate::status::status;
use chapchoo_protocol::control::{self as wire, peer_relay_client::PeerRelayClient};
use chapchoo_service::{BearerSecret, ComputersService, ResolvedRoute};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::Semaphore, time::Instant};
use tonic::{
    Request, Status,
    transport::{Channel, Endpoint},
};

#[derive(Debug, Clone)]
pub struct PeerDelivery {
    service: Arc<ComputersService>,
    channels: Arc<Mutex<BTreeMap<String, Channel>>>,
    local: Arc<Mutex<Option<(String, Channel)>>>,
    pending: Arc<Semaphore>,
}
impl PeerDelivery {
    pub async fn converge(
        &self,
        mut value: wire::ProviderConvergence,
        deadline: Instant,
    ) -> Result<wire::ProviderInvocationResult, Status> {
        let _permit = self
            .pending
            .try_acquire()
            .map_err(|_| Status::resource_exhausted("Peer pending capacity is exhausted"))?;
        for _ in 0..2 {
            let route = tokio::time::timeout_at(
                deadline,
                self.service.resolve_provider_route(&value.provider_id),
            )
            .await
            .map_err(|_| Status::deadline_exceeded("Reconciliation deadline expired"))?
            .map_err(status)?;
            let Some(route) = route else {
                return Ok(wire::ProviderInvocationResult {
                    outcome: wire::ForwardingOutcome::Unavailable as i32,
                    result: None,
                });
            };
            value.owner = Some(owner(&route));
            value.remaining_ms = deadline
                .saturating_duration_since(Instant::now())
                .as_millis()
                .min(10_000) as u32;
            if value.remaining_ms == 0 {
                return Err(Status::deadline_exceeded("Reconciliation deadline expired"));
            }
            let mut client = PeerRelayClient::new(self.channel(&route)?)
                .max_decoding_message_size(64 * 1024)
                .max_encoding_message_size(64 * 1024);
            let mut request = Request::new(value.clone());
            request.set_timeout(deadline.saturating_duration_since(Instant::now()));
            let result =
                match tokio::time::timeout_at(deadline, client.converge_provider(request)).await {
                    Ok(Ok(result)) => result.into_inner(),
                    _ => {
                        return Ok(wire::ProviderInvocationResult {
                            outcome: wire::ForwardingOutcome::Uncertain as i32,
                            result: None,
                        });
                    }
                };
            if result.outcome != wire::ForwardingOutcome::NotOwner as i32 {
                return Ok(result);
            }
        }
        Ok(wire::ProviderInvocationResult {
            outcome: wire::ForwardingOutcome::NotOwner as i32,
            result: None,
        })
    }
    pub fn new(service: Arc<ComputersService>) -> Self {
        Self {
            service,
            channels: Arc::new(Mutex::new(BTreeMap::new())),
            local: Arc::new(Mutex::new(None)),
            pending: Arc::new(Semaphore::new(256)),
        }
    }
    /// Install the same-handler local memory channel for this process identity.
    pub fn register_local(&self, instance_id: String, channel: Channel) -> Result<(), Status> {
        let mut local = self
            .local
            .lock()
            .map_err(|_| Status::unavailable("Peer channels are unavailable"))?;
        if local.is_some() {
            return Err(Status::already_exists(
                "Local peer channel is already registered",
            ));
        }
        *local = Some((instance_id, channel));
        Ok(())
    }
    fn channel(&self, route: &ResolvedRoute) -> Result<Channel, Status> {
        if let Some((id, channel)) = self
            .local
            .lock()
            .map_err(|_| Status::unavailable("Peer channels are unavailable"))?
            .as_ref()
            && id == &route.component.control_id
        {
            return Ok(channel.clone());
        }
        let mut channels = self
            .channels
            .lock()
            .map_err(|_| Status::unavailable("Peer channels are unavailable"))?;
        if let Some(channel) = channels.get(&route.component.control_id) {
            return Ok(channel.clone());
        }
        if channels.len() >= 64 {
            // Channels are reusable transport, not ownership authority. Evicting
            // a cache entry never closes a channel cloned by an active call.
            if let Some(id) = channels.keys().next().cloned() {
                channels.remove(&id);
            }
        }
        let endpoint = Endpoint::from_shared(route.control.endpoint.clone())
            .map_err(|_| Status::unavailable("Peer endpoint is invalid"))?
            .connect_timeout(Duration::from_secs(3))
            .buffer_size(32)
            .concurrency_limit(32);
        let channel = endpoint.connect_lazy();
        channels.insert(route.component.control_id.clone(), channel.clone());
        Ok(channel)
    }
    pub async fn provider(
        &self,
        mut invocation: wire::ProviderInvocation,
        deadline: Instant,
    ) -> Result<wire::ProviderInvocationResult, Status> {
        let _permit = self
            .pending
            .try_acquire()
            .map_err(|_| Status::resource_exhausted("Peer pending capacity is exhausted"))?;
        let deadline = deadline.min(Instant::now() + Duration::from_secs(10));
        for _ in 0..2 {
            let route = tokio::time::timeout_at(
                deadline,
                self.service.resolve_provider_route(&invocation.provider_id),
            )
            .await
            .map_err(|_| Status::deadline_exceeded("Peer deadline expired"))?
            .map_err(status)?;
            let Some(route) = route else {
                return Ok(wire::ProviderInvocationResult {
                    outcome: wire::ForwardingOutcome::Unavailable as i32,
                    result: None,
                });
            };
            let command = invocation
                .command
                .as_mut()
                .ok_or_else(|| Status::invalid_argument("Provider command is required"))?;
            update(
                &mut command.identity,
                &mut command.remaining_ms,
                &route,
                deadline,
            )?;
            invocation.owner = Some(owner(&route));
            let mut client = PeerRelayClient::new(self.channel(&route)?)
                .max_decoding_message_size(64 * 1024)
                .max_encoding_message_size(64 * 1024);
            let mut request = Request::new(invocation.clone());
            request.set_timeout(deadline.saturating_duration_since(Instant::now()));
            let result =
                match tokio::time::timeout_at(deadline, client.invoke_provider(request)).await {
                    Ok(Ok(response)) => response.into_inner(),
                    // Native status (including timeout/unavailable) cannot prove the
                    // receiver did not forward. Never reconnect/replay this call.
                    _ => {
                        return Ok(wire::ProviderInvocationResult {
                            outcome: wire::ForwardingOutcome::Uncertain as i32,
                            result: None,
                        });
                    }
                };
            if result.outcome != wire::ForwardingOutcome::NotOwner as i32 {
                return Ok(result);
            }
        }
        Ok(wire::ProviderInvocationResult {
            outcome: wire::ForwardingOutcome::NotOwner as i32,
            result: None,
        })
    }
    pub async fn connector(
        &self,
        mut invocation: wire::ConnectorInvocation,
        deadline: Instant,
    ) -> Result<wire::ConnectorInvocationResult, Status> {
        let _permit = self
            .pending
            .try_acquire()
            .map_err(|_| Status::resource_exhausted("Peer pending capacity is exhausted"))?;
        let deadline = deadline.min(Instant::now() + Duration::from_secs(10));
        for _ in 0..2 {
            let route = tokio::time::timeout_at(
                deadline,
                self.service
                    .resolve_connector_route(&invocation.computer_id),
            )
            .await
            .map_err(|_| Status::deadline_exceeded("Peer deadline expired"))?
            .map_err(status)?;
            let Some(route) = route else {
                return Ok(wire::ConnectorInvocationResult {
                    outcome: wire::ForwardingOutcome::Unavailable as i32,
                    result: None,
                });
            };
            let command = invocation
                .command
                .as_mut()
                .ok_or_else(|| Status::invalid_argument("Connector command is required"))?;
            update(
                &mut command.identity,
                &mut command.remaining_ms,
                &route,
                deadline,
            )?;
            invocation.owner = Some(owner(&route));
            let mut client = PeerRelayClient::new(self.channel(&route)?)
                .max_decoding_message_size(64 * 1024)
                .max_encoding_message_size(64 * 1024);
            let mut request = Request::new(invocation.clone());
            request.set_timeout(deadline.saturating_duration_since(Instant::now()));
            let result =
                match tokio::time::timeout_at(deadline, client.invoke_connector(request)).await {
                    Ok(Ok(response)) => response.into_inner(),
                    _ => {
                        return Ok(wire::ConnectorInvocationResult {
                            outcome: wire::ForwardingOutcome::Uncertain as i32,
                            result: None,
                        });
                    }
                };
            if result.outcome != wire::ForwardingOutcome::NotOwner as i32 {
                return Ok(result);
            }
        }
        Ok(wire::ConnectorInvocationResult {
            outcome: wire::ForwardingOutcome::NotOwner as i32,
            result: None,
        })
    }
}
fn owner(route: &ResolvedRoute) -> wire::ExpectedOwner {
    wire::ExpectedOwner {
        instance_id: route.component.control_id.clone(),
        connection_id: route.component.connection_id.clone(),
    }
}
fn update(
    identity: &mut Option<wire::CallIdentity>,
    remaining: &mut u32,
    route: &ResolvedRoute,
    deadline: Instant,
) -> Result<(), Status> {
    *remaining = deadline
        .saturating_duration_since(Instant::now())
        .as_millis()
        .min(10_000) as u32;
    if *remaining == 0 {
        return Err(Status::deadline_exceeded("Peer deadline expired"));
    }
    let identity = identity
        .as_mut()
        .ok_or_else(|| Status::invalid_argument("Call identity is required"))?;
    identity.call_id = BearerSecret::generate()
        .map_err(|_| Status::internal("Call identity creation failed"))?
        .digest()
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    identity.connection_id = route.component.connection_id.clone();
    Ok(())
}
