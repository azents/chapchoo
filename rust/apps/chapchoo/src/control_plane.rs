//! Owned Control lifetime. Listener binding precedes directory publication.
use crate::AppError;
use crate::completion::complete;
use chapchoo_rpc::{
    ComponentAuthenticationServices, ComponentConnectionServices, MemoryIncoming, PeerDelivery,
    PeerServices, memory_transport,
};
use chapchoo_service::ComputersService;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct ControlPlane {
    service: Arc<ComputersService>,
    id: String,
    pub authentication: ComponentAuthenticationServices,
    pub connections: ComponentConnectionServices,
    pub peer: PeerServices,
    // Keep the local acceptor alive until the owned Control is shut down.
    _local_peer: tonic::transport::Channel,
    delivery: PeerDelivery,
}
impl ControlPlane {
    pub fn conversation_delivery(&self) -> PeerDelivery {
        self.delivery.clone()
    }
    pub async fn new(
        service: Arc<ComputersService>,
        endpoint: &str,
        admission: Arc<AtomicBool>,
    ) -> Result<(Self, MemoryIncoming), AppError> {
        let registration = service.register_control(endpoint).await?;
        let connections = ComponentConnectionServices::new(
            service.clone(),
            registration.id.clone(),
            admission.clone(),
        );
        let authentication = ComponentAuthenticationServices::new(
            service.clone(),
            registration.id.clone(),
            connections.clone(),
        )
        .with_admission(admission);
        let peer = PeerServices::new(service.clone(), connections.clone());
        let (channel, incoming) = memory_transport();
        let delivery = PeerDelivery::new(service.clone());
        delivery.register_local(registration.id.clone(), channel.clone())?;
        Ok((
            Self {
                service,
                id: registration.id,
                authentication,
                connections,
                peer,
                _local_peer: channel,
                delivery,
            },
            incoming,
        ))
    }
    pub fn reconciler(
        &self,
        admission: Arc<AtomicBool>,
        stop: CancellationToken,
    ) -> impl Future<Output = Result<(), AppError>> + Send + 'static {
        let connections = self.connections.clone();
        let delivery = self.delivery.clone();
        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(15));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _=stop.cancelled()=>return Ok(()),
                    _=interval.tick()=>{
                        if !admission.load(Ordering::Acquire){continue;}
                        tokio::select!{_=stop.cancelled()=>return Ok(()),result=connections.reconcile_once(&delivery)=>{result?;}}
                    }
                }
            }
        }
    }
    pub fn monitor(
        &self,
        admission: Arc<AtomicBool>,
        stop: CancellationToken,
    ) -> impl Future<Output = Result<(), AppError>> + Send + 'static {
        let service = self.service.clone();
        let id = self.id.clone();
        let connections = self.connections.clone();
        let heartbeat = Duration::from_millis(u64::from(service.policy().heartbeat_ms));
        async move {
            let mut interval =
                tokio::time::interval_at(tokio::time::Instant::now() + heartbeat, heartbeat);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _=stop.cancelled()=>return Ok(()),
                    _=interval.tick()=>{
                        let result=service.renew_control(&id).await;
                        if result.is_err() || (admission.load(Ordering::Acquire)&&connections.ensure_healthy().is_err()) {
                            admission.store(false,Ordering::Release);
                            return Err(result.err().map(|e|Box::new(e) as AppError).unwrap_or_else(||Box::new(crate::config::ConfigError::ComponentExit)));
                        }
                    }
                }
            }
        }
    }
    pub async fn shutdown(self) -> Result<(), AppError> {
        let directory = self.service.drain_control(&self.id).await;
        let streams = self.connections.drain().await;
        let withdrawal = if directory.is_ok() {
            self.service
                .withdraw_drained_control_connections(&self.id)
                .await
                .map(|_| ())
        } else {
            Ok(())
        };
        let primary = complete(directory.map_err(Into::into), streams.map_err(Into::into));
        complete(primary, withdrawal.map_err(Into::into))
    }
}
