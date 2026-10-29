use chapchoo_crypto::PasswordCost;
use chapchoo_rpc::RpcService;
use chapchoo_service::{AccountsService, Mailer, PasswordWork};
use std::{
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
    config::{ApplicationConfig, BackendConfig, ConfigError, Role, ServeOptions},
    health::HealthService,
};

pub async fn serve(config: BackendConfig, options: ServeOptions) -> Result<(), AppError> {
    if matches!(options.role, Role::Both) && options.general_listen == options.admin_listen {
        return Err(ConfigError::SameListener.into());
    }
    let admission = Arc::new(AtomicBool::new(false));
    let stop = CancellationToken::new();
    let mut tasks = JoinSet::new();
    let mut signal = Termination::new()?;
    // Listeners are bound before task creation so partial bind failure drops every socket.
    let startup = async {
        let application = ApplicationConfig::load()?;
        let store = config.connect().await?;
        let mailer: Option<Arc<dyn Mailer>> = match application.mail {
            Some(mail) => Some(Arc::new(
                chapchoo_mail::SesMailer::new(mail.sender, mail.region, mail.endpoint).await,
            )),
            None => None,
        };
        let service = Arc::new(AccountsService::new(
            store,
            PasswordWork::new(application.password_capacity, PasswordCost::default())?,
            mailer,
            application.accounts,
        )?);
        let general = if matches!(options.role, Role::General | Role::Both) {
            Some(TcpListener::bind(options.general_listen).await?)
        } else {
            None
        };
        let admin = if matches!(options.role, Role::Admin | Role::Both) {
            Some(TcpListener::bind(options.admin_listen).await?)
        } else {
            None
        };
        Ok::<_, AppError>((service, general, admin))
    };
    let started = tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(60), startup) => {
            result.map_err(|_| ConfigError::StartupTimeout)??
        }
        result = signal.wait() => {
            result?;
            tracing::info!("Termination requested during backend startup");
            return Ok(());
        }
    };
    let (service, general, admin) = started;
    for (listener, general_role) in general
        .into_iter()
        .map(|socket| (socket, true))
        .chain(admin.into_iter().map(|socket| (socket, false)))
    {
        let rpc = RpcService::new(service.clone()).with_admission(admission.clone());
        let health = HealthServer::new(HealthService {
            config: config.clone(),
            admission: admission.clone(),
            business_service: if general_role {
                "chapchoo.accounts.General"
            } else {
                "chapchoo.accounts.Admin"
            },
        });
        let shutdown = stop.clone();
        tasks.spawn(async move {
            let server = Server::builder()
                .add_service(health)
                .add_service(rpc.auth_server());
            if general_role {
                server
                    .add_service(rpc.general_server())
                    .serve_with_incoming_shutdown(
                        TcpListenerStream::new(listener),
                        shutdown.cancelled_owned(),
                    )
                    .await
            } else {
                server
                    .add_service(rpc.admin_server())
                    .serve_with_incoming_shutdown(
                        TcpListenerStream::new(listener),
                        shutdown.cancelled_owned(),
                    )
                    .await
            }
        });
    }
    admission.store(true, Ordering::Release);
    tracing::info!("Selected backend roles are ready");
    let failure = tokio::select! {
        result = signal.wait() => result.err().map(|error| Box::new(error) as AppError),
        exited = tasks.join_next() => {
            match exited {
                Some(Ok(Err(error))) => Some(Box::new(error) as AppError),
                Some(Err(error)) => Some(Box::new(error) as AppError),
                Some(Ok(Ok(()))) | None => Some(Box::new(ConfigError::ComponentExit) as AppError),
            }
        }
    };
    admission.store(false, Ordering::Release);
    stop.cancel();
    let drained = tokio::time::timeout(Duration::from_secs(20), async {
        let mut failure = None;
        while let Some(result) = tasks.join_next().await {
            let error = match result {
                Err(error) => Some(Box::new(error) as AppError),
                Ok(Err(error)) => Some(Box::new(error) as AppError),
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
    })
    .await;
    let cleanup = match drained {
        Ok(result) => result,
        Err(_) => {
            tasks.abort_all();
            let forced = tokio::time::timeout(Duration::from_secs(5), async {
                let mut failure = None;
                while let Some(result) = tasks.join_next().await {
                    let error = match result {
                        Err(error) if error.is_cancelled() => None,
                        Err(error) => Some(Box::new(error) as AppError),
                        Ok(Err(error)) => Some(Box::new(error) as AppError),
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
            })
            .await;
            // A forced drain is a failure even if all owned async tasks are reaped.
            if let Ok(Err(error)) = forced {
                return Err(error);
            }
            Err(ConfigError::CleanupTimeout.into())
        }
    };
    cleanup?;
    if let Some(error) = failure {
        return Err(error);
    }
    tracing::info!("Selected backend roles stopped");
    Ok(())
}

#[cfg(unix)]
struct Termination {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
}

#[cfg(unix)]
impl Termination {
    fn new() -> Result<Self, std::io::Error> {
        use tokio::signal::unix::{SignalKind, signal};
        Ok(Self {
            interrupt: signal(SignalKind::interrupt())?,
            terminate: signal(SignalKind::terminate())?,
        })
    }

    async fn wait(&mut self) -> Result<(), std::io::Error> {
        tokio::select! {
            _ = self.interrupt.recv() => {},
            _ = self.terminate.recv() => {},
        }
        Ok(())
    }
}

#[cfg(not(unix))]
struct Termination;

#[cfg(not(unix))]
impl Termination {
    fn new() -> Result<Self, std::io::Error> {
        Ok(Self)
    }

    async fn wait(&mut self) -> Result<(), std::io::Error> {
        tokio::signal::ctrl_c().await
    }
}
