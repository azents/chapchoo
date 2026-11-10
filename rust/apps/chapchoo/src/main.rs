//! Foreground composition root and local operator commands.
mod completion;
mod config;
mod connector;
mod control_plane;
mod diagnostics;
mod health;
mod provider;
mod provider_auth;
mod provider_credential;
mod runtime;
#[cfg(test)]
#[path = "../test_support.rs"]
mod test_support;
mod web;

use clap::Parser;
use config::{Cli, Command};
use std::{backtrace::Backtrace, error::Error, process::ExitCode, time::Duration};

type AppError = Box<dyn Error + Send + Sync>;

fn main() -> ExitCode {
    diagnostics::initialize();
    let cli = Cli::parse();
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => {
            tracing::error!("Could not create the async runtime");
            return ExitCode::FAILURE;
        }
    };
    let result = runtime.block_on(execute(cli));
    // Started blocking password/readiness work cannot be aborted. Do not wait
    // indefinitely after the owned async drain has reached its finite deadline.
    runtime.shutdown_timeout(Duration::ZERO);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let mut depth = 0;
            let mut source = error.source();
            while let Some(value) = source {
                depth += 1;
                source = value.source();
            }
            // Dependency source messages may contain SQL, endpoints or credentials.
            // Preserve them in the typed error, but publish only safe context.
            if let Some(native) = error.downcast_ref::<chapchoo_docker::Error>() {
                tracing::error!(
                    engine = ?native.engine_diagnostic(),
                    native_docker = %native,
                    "Native Docker command failed"
                );
            }
            if let Some(status) = error.downcast_ref::<tonic::Status>() {
                tracing::error!(code = ?status.code(), "Control RPC command failed");
            }
            tracing::error!(
                source_depth = depth,
                configuration = ?error.downcast_ref::<config::ConfigError>(),
                backtrace = %Backtrace::force_capture(),
                "Chapchoo command failed; verify configuration and service prerequisites"
            );
            ExitCode::FAILURE
        }
    }
}

async fn execute(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Command::Prepare => {
            let config = config::BackendConfig::load()?;
            config.prepare().await?;
            tracing::info!("Schema preparation completed");
        }
        Command::GrantAdmin { email } => {
            let config = config::BackendConfig::load()?;
            let email = chapchoo_domain::EmailAddress::parse(&email)?;
            let store = config.connect().await?;
            store.grant_administrator(&email).await?;
            tracing::info!("Administrator assignment completed for the existing account");
        }
        Command::Serve(options) => {
            let config = if options.groups.is_empty() {
                None
            } else {
                Some(config::BackendConfig::load()?)
            };
            runtime::serve(config, options).await?;
        }
        Command::ComputerProvider(options) => {
            let owner = provider_auth::ProviderAuth::from_process()?;
            let stop = tokio_util::sync::CancellationToken::new();
            let mut signal = runtime::Termination::new()?;
            let task = provider::run(options, owner, stop.clone());
            tokio::pin!(task);
            tokio::select! {
                result=&mut task=>result?,
                result=signal.wait()=>{
                    result?;
                    stop.cancel();
                    tokio::time::timeout(Duration::from_secs(120),task).await??;
                }
            }
        }
        Command::DockerProvider(options) => {
            let owner = provider_auth::ProviderAuth::from_process()?;
            let stop = tokio_util::sync::CancellationToken::new();
            let mut signal = runtime::Termination::new()?;
            let task = provider::run_docker(options, owner, stop.clone());
            tokio::pin!(task);
            tokio::select! {
                result=&mut task=>result?,
                result=signal.wait()=>{
                    result?;
                    stop.cancel();
                    tokio::time::timeout(Duration::from_secs(120),task).await??;
                }
            }
        }
        Command::ComputerConnector(options) => {
            let initial = config::connector_initial_key()?;
            let stop = tokio_util::sync::CancellationToken::new();
            let mut signal = runtime::Termination::new()?;
            let native = chapchoo_harness::NativeOwner::new(
                chapchoo_harness::NativeStorage::open(&options.saved_root)?,
            )?;
            let task = connector::run(
                &options.endpoint,
                options.session_id,
                initial,
                stop.clone(),
                native,
            );
            tokio::pin!(task);
            tokio::select! {
                result=&mut task=>result?,
                result=signal.wait()=>{
                    result?;
                    stop.cancel();
                    tokio::time::timeout(Duration::from_secs(15),task).await??;
                }
            }
        }
        Command::ProviderCredential(options) => provider_credential::run(options).await?,
    }
    Ok(())
}
