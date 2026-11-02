use chapchoo_crypto::PasswordCost;
use chapchoo_rpc::{AccountServices, ComputerServices, ServiceGroup, ServiceGroups};
use chapchoo_service::{AccountsService, ComputersService, Mailer, PasswordWork};
use std::{
    error::Error,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{net::TcpListener, task::JoinSet};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tonic_health::pb::health_server::HealthServer;

use crate::{
    AppError,
    config::{ApplicationConfig, BackendConfig, ConfigError, ServeOptions, WebApp, web_configs},
    control_plane::ControlPlane,
    health::HealthService,
    web::{WebChild, prepare_ownership},
};

type BackendTasks = JoinSet<Result<(), AppError>>;

/// Preserve both failures without exposing native source messages through Debug.
struct FailedCleanup {
    primary: AppError,
    cleanup: AppError,
}

impl fmt::Debug for FailedCleanup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("FailedCleanup([REDACTED])")
    }
}

impl fmt::Display for FailedCleanup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Selected-component failure was followed by owned cleanup failure")
    }
}

impl Error for FailedCleanup {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Keep the cleanup error reachable through its concrete field as well;
        // source traversal must never format either native error for operators.
        let _ = &self.cleanup;
        Some(self.primary.as_ref())
    }
}

pub async fn serve(config: Option<BackendConfig>, options: ServeOptions) -> Result<(), AppError> {
    if options.groups.is_empty() && options.web.is_none() {
        return Err(ConfigError::NoComponents.into());
    }
    let groups = ServiceGroups::new(options.groups.iter().map(|group| group.domain()));
    if options.peer_listen == Some(options.listen) {
        return Err(ConfigError::SameListener.into());
    }
    if options.peer_listen.is_some() != options.peer_advertise.is_some()
        || (options.peer_listen.is_some() && !groups.contains(ServiceGroup::Control))
    {
        return Err(ConfigError::PeerConfiguration.into());
    }
    let web = web_configs(&options)?;
    if web.len() == 2 && options.general_web_listen == options.admin_web_listen {
        return Err(ConfigError::SameListener.into());
    }
    if !web.is_empty() {
        prepare_ownership()?;
    }
    let admission = Arc::new(AtomicBool::new(false));
    let stop = CancellationToken::new();
    let mut tasks = JoinSet::new();
    let mut general_web = None;
    let mut admin_web = None;
    let mut control_plane = None;
    let mut signal = Termination::new()?;
    let startup = async {
        if let Some(config) = &config {
            start_backends(
                config,
                &options,
                &admission,
                &stop,
                &mut tasks,
                &mut control_plane,
            )
            .await?;
        }
        for config in web {
            let app = config.app;
            let child = WebChild::spawn(config)?;
            match app {
                WebApp::General => general_web = Some(child),
                WebApp::Admin => admin_web = Some(child),
            }
        }
        // Both children have explicit ownership before waiting for either. A
        // partial failure can therefore clean up the complete started set.
        tokio::select! {
            ready = ready_web_pair(&mut general_web, &mut admin_web) => ready?,
            result = tasks.join_next(), if !tasks.is_empty() => {
                return match result {
                    Some(result) => component_result(result),
                    None => Err(ConfigError::ComponentExit.into()),
                };
            }
        }
        if let Some(result) = tasks.try_join_next() {
            return component_result(result);
        }
        for child in [general_web.as_mut(), admin_web.as_mut()]
            .into_iter()
            .flatten()
        {
            child.ensure_running()?;
        }
        Ok::<_, AppError>(())
    };
    enum Startup {
        Ready,
        Terminated,
        Failed(AppError),
    }
    let startup = tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(60), startup) => {
            match result {
                Ok(Ok(())) => Startup::Ready,
                Ok(Err(error)) => Startup::Failed(error),
                Err(_) => Startup::Failed(ConfigError::StartupTimeout.into()),
            }
        }
        result = signal.wait() => {
            match result {
                Ok(()) => Startup::Terminated,
                Err(error) => Startup::Failed(error.into()),
            }
        }
    };
    let failure = match startup {
        Startup::Ready => {
            admission.store(true, Ordering::Release);
            if config.is_some() {
                tracing::info!("Selected backend service groups are ready");
            }
            tracing::info!("Selected components are ready");
            tokio::select! {
                result = signal.wait() => result.err().map(|error| Box::new(error) as AppError),
                result = tasks.join_next(), if !tasks.is_empty() => {
                    Some(match result {
                        Some(result) => component_result(result).err().unwrap_or_else(|| ConfigError::ComponentExit.into()),
                        None => ConfigError::ComponentExit.into(),
                    })
                }
                result = wait_web(&mut general_web), if general_web.is_some() => {
                    Some(web_exit(result))
                }
                result = wait_web(&mut admin_web), if admin_web.is_some() => {
                    Some(web_exit(result))
                }
            }
        }
        Startup::Terminated => {
            tracing::info!("Termination requested during selected-component startup");
            None
        }
        Startup::Failed(error) => {
            if matches!(
                error.downcast_ref::<ConfigError>(),
                Some(ConfigError::StartupTimeout)
            ) {
                let source = general_web
                    .as_mut()
                    .and_then(WebChild::take_probe_failure)
                    .or_else(|| admin_web.as_mut().and_then(WebChild::take_probe_failure));
                Some(source.unwrap_or(error))
            } else {
                Some(error)
            }
        }
    };
    admission.store(false, Ordering::Release);
    stop.cancel();
    if let Some(control) = control_plane.take() {
        tasks.spawn(control.shutdown());
    }
    let cleanup = cleanup(&mut tasks, &mut general_web, &mut admin_web).await;
    match (failure, cleanup) {
        (Some(primary), Err(cleanup)) => Err(Box::new(FailedCleanup { primary, cleanup })),
        (Some(error), Ok(())) | (None, Err(error)) => Err(error),
        (None, Ok(())) => {
            tracing::info!("Selected components stopped");
            Ok(())
        }
    }
}

async fn start_backends(
    config: &BackendConfig,
    options: &ServeOptions,
    admission: &Arc<AtomicBool>,
    stop: &CancellationToken,
    tasks: &mut BackendTasks,
    control: &mut Option<ControlPlane>,
) -> Result<(), AppError> {
    let application = ApplicationConfig::load()?;
    let store = config.connect().await?;
    let mailer: Option<Arc<dyn Mailer>> = match application.mail {
        Some(mail) => Some(Arc::new(chapchoo_mail::SmtpMailer::new(
            mail.sender,
            mail.host,
            mail.port,
            mail.security,
            mail.credentials,
        )?)),
        None => None,
    };
    let service = Arc::new(AccountsService::new(
        store.clone(),
        PasswordWork::new(application.password_capacity, PasswordCost::default())?,
        mailer,
        application.accounts,
    )?);
    // All selected sockets are owned before a Control can publish its endpoint.
    let listener = TcpListener::bind(options.listen).await?;
    let peer_listener = if let Some(address) = options.peer_listen {
        Some(TcpListener::bind(address).await?)
    } else {
        None
    };
    let groups = ServiceGroups::new(options.groups.iter().map(|group| group.domain()));
    let app = Arc::new(
        ComputersService::new(store, application.system_owner)
            .with_policy(application.computers)?,
    );
    let computers = ComputerServices::new(app.clone()).with_admission(admission.clone());
    if groups.contains(ServiceGroup::Control) {
        let (plane, incoming) = ControlPlane::new(
            app,
            options
                .peer_advertise
                .as_deref()
                .unwrap_or("http://chapchoo.memory"),
            admission.clone(),
        )
        .await?;
        *control = Some(plane);
        let plane = control.as_ref().ok_or(ConfigError::ComponentExit)?;
        let peer = plane.peer.clone();
        let shutdown = stop.clone();
        tasks.spawn(async move {
            Server::builder()
                .add_service(peer.server())
                .serve_with_incoming_shutdown(incoming, shutdown.cancelled_owned())
                .await?;
            Ok(())
        });
        if let Some(listener) = peer_listener {
            let peer = plane.peer.clone();
            let shutdown = stop.clone();
            tasks.spawn(async move {
                Server::builder()
                    .add_service(peer.server())
                    .serve_with_incoming_shutdown(
                        TcpListenerStream::new(listener),
                        shutdown.cancelled_owned(),
                    )
                    .await?;
                Ok(())
            });
        }
        tasks.spawn(plane.monitor(admission.clone(), stop.clone()));
        tasks.spawn(plane.reconciler(admission.clone(), stop.clone()));
    }
    let rpc = AccountServices::new(service).with_admission(admission.clone());
    let components = control
        .as_ref()
        .map(|plane| (&plane.authentication, &plane.connections));
    let routes = groups.routes(Some(&rpc), Some(&computers), components)?;
    let health = HealthServer::new(HealthService {
        config: config.clone(),
        admission: admission.clone(),
        business_services: groups.service_names().into(),
    });
    let shutdown = stop.clone();
    tasks.spawn(async move {
        Server::builder()
            .add_routes(routes.add_service(health))
            .serve_with_incoming_shutdown(
                TcpListenerStream::new(listener),
                shutdown.cancelled_owned(),
            )
            .await?;
        Ok(())
    });
    Ok(())
}

fn component_result(
    result: Result<Result<(), AppError>, tokio::task::JoinError>,
) -> Result<(), AppError> {
    match result {
        Ok(Ok(())) => Err(ConfigError::ComponentExit.into()),
        Ok(Err(error)) => Err(error),
        Err(error) => Err(error.into()),
    }
}

fn web_exit(result: Result<std::process::ExitStatus, AppError>) -> AppError {
    match result {
        Ok(status) => {
            tracing::error!(
                code = status.code(),
                "A selected web process exited unexpectedly"
            );
            ConfigError::ComponentExit.into()
        }
        Err(error) => error,
    }
}

async fn wait_web(child: &mut Option<WebChild>) -> Result<std::process::ExitStatus, AppError> {
    match child {
        Some(child) => child.wait().await,
        None => std::future::pending().await,
    }
}

async fn ready_web(child: &mut Option<WebChild>) -> Result<(), AppError> {
    if let Some(child) = child {
        child.ready().await?;
        tracing::debug!(app = ?child.app, "Selected web process is ready");
    }
    Ok(())
}

async fn ready_web_pair(
    general: &mut Option<WebChild>,
    admin: &mut Option<WebChild>,
) -> Result<(), AppError> {
    // A completed readiness future is not a continuing liveness observation.
    // Keep the first-ready child's exit supervised while the other starts.
    // Retain the pending future: recreating it could poll an already-consumed
    // Ready oneshot after cancellation in the middle of its health probe.
    let mut general_ready = Box::pin(ready_web(general));
    let mut admin_ready = Box::pin(ready_web(admin));
    tokio::select! {
        result = general_ready.as_mut() => {
            drop(general_ready);
            result?;
            tokio::select! {
                biased;
                exit = wait_web(general), if general.is_some() => Err(web_exit(exit)),
                result = admin_ready.as_mut() => result,
            }
        }
        result = admin_ready.as_mut() => {
            drop(admin_ready);
            result?;
            tokio::select! {
                biased;
                exit = wait_web(admin), if admin.is_some() => Err(web_exit(exit)),
                result = general_ready.as_mut() => result,
            }
        }
    }
}

async fn reap_web(child: &mut Option<WebChild>) -> Result<(), AppError> {
    if let Some(child) = child {
        child.reap().await?;
    }
    Ok(())
}

async fn drain(tasks: &mut BackendTasks) -> Result<(), AppError> {
    let mut failure = None;
    while let Some(result) = tasks.join_next().await {
        let error = match result {
            Err(error) if error.is_cancelled() => None,
            Err(error) => Some(Box::new(error) as AppError),
            Ok(Err(error)) => Some(error),
            Ok(Ok(())) => None,
        };
        if failure.is_none() {
            failure = error;
        }
    }
    match failure {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

async fn join_owned(
    tasks: &mut BackendTasks,
    general: &mut Option<WebChild>,
    admin: &mut Option<WebChild>,
) -> Result<(), AppError> {
    let (backend, general, admin) = tokio::join!(drain(tasks), reap_web(general), reap_web(admin));
    backend?;
    general?;
    admin?;
    Ok(())
}

async fn cleanup(
    tasks: &mut BackendTasks,
    general: &mut Option<WebChild>,
    admin: &mut Option<WebChild>,
) -> Result<(), AppError> {
    let mut signal_failure = None;
    for child in [general.as_ref(), admin.as_ref()].into_iter().flatten() {
        if let Err(error) = child.terminate() {
            signal_failure.get_or_insert(error);
        }
    }
    let graceful =
        tokio::time::timeout(Duration::from_secs(20), join_owned(tasks, general, admin)).await;
    if let Ok(result) = graceful {
        result?;
        return match signal_failure {
            Some(error) => Err(error),
            None => Ok(()),
        };
    }
    for child in [general.as_ref(), admin.as_ref()].into_iter().flatten() {
        if let Err(error) = child.force() {
            signal_failure.get_or_insert(error);
        }
    }
    tasks.abort_all();
    let forced =
        tokio::time::timeout(Duration::from_secs(5), join_owned(tasks, general, admin)).await;
    if let Some(error) = signal_failure {
        return Err(error);
    }
    if let Ok(Err(error)) = forced {
        return Err(error);
    }
    // Forced cleanup or an unreaped child is never a successful graceful stop.
    Err(ConfigError::CleanupTimeout.into())
}

#[cfg(unix)]
pub(crate) struct Termination {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
}

#[cfg(unix)]
impl Termination {
    pub(crate) fn new() -> Result<Self, std::io::Error> {
        use tokio::signal::unix::{SignalKind, signal};
        Ok(Self {
            interrupt: signal(SignalKind::interrupt())?,
            terminate: signal(SignalKind::terminate())?,
        })
    }

    pub(crate) async fn wait(&mut self) -> Result<(), std::io::Error> {
        tokio::select! {
            _ = self.interrupt.recv() => {},
            _ = self.terminate.recv() => {},
        }
        Ok(())
    }
}

#[cfg(not(unix))]
pub(crate) struct Termination;

#[cfg(not(unix))]
impl Termination {
    pub(crate) fn new() -> Result<Self, std::io::Error> {
        Ok(Self)
    }

    pub(crate) async fn wait(&mut self) -> Result<(), std::io::Error> {
        tokio::signal::ctrl_c().await
    }
}
