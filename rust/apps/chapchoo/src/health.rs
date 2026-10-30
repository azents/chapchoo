use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tonic::{Request, Response, Status};
use tonic_health::pb::{
    HealthCheckRequest, HealthCheckResponse, health_check_response::ServingStatus,
    health_server::Health,
};

use crate::config::BackendConfig;

#[derive(Debug, Clone)]
pub struct HealthService {
    pub config: BackendConfig,
    pub admission: Arc<AtomicBool>,
    pub business_service: &'static str,
}

#[tonic::async_trait]
impl Health for HealthService {
    async fn check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let service = request.into_inner().service;
        if service == "chapchoo.liveness" {
            return Ok(Response::new(HealthCheckResponse {
                status: ServingStatus::Serving as i32,
            }));
        }
        if !service.is_empty()
            && service != self.business_service
            && service != "chapchoo.accounts.Auth"
        {
            return Err(Status::not_found("Unknown health service"));
        }
        let ready = self.admission.load(Ordering::Acquire)
            && matches!(
                tokio::time::timeout(Duration::from_secs(5), self.config.ready()).await,
                Ok(Ok(()))
            )
            && self.admission.load(Ordering::Acquire);
        Ok(Response::new(HealthCheckResponse {
            status: if ready {
                ServingStatus::Serving
            } else {
                ServingStatus::NotServing
            } as i32,
        }))
    }

    type WatchStream =
        Pin<Box<dyn tokio_stream::Stream<Item = Result<HealthCheckResponse, Status>> + Send>>;

    async fn watch(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        Err(Status::unimplemented("Use unary health checks"))
    }
}
