//! Once-only native container stages. Opaque consumed tokens separate effects so
//! the Provider rechecks exact Control authority between every mutating stage.
mod profile;
use crate::{Docker, Error, Observation, Presence, volumes::not_found};
use bollard::{
    models::{
        ContainerInspectResponse, ContainerState, ContainerStateStatusEnum as Status,
        ContainerUpdateBody, RestartPolicy, RestartPolicyNameEnum,
    },
    query_parameters::{
        CreateContainerOptionsBuilder, RemoveContainerOptionsBuilder, StopContainerOptionsBuilder,
    },
};
use chapchoo_domain::{DockerContainerId, ManagedResourceKind, NativeReservation, NativeScope};
use chapchoo_protocol::control::{self as wire, managed_resource::Native};
pub use profile::Bootstrap;
use std::{collections::BTreeMap, fmt, sync::Mutex, time::Duration};

const REQUEST: Duration = Duration::from_secs(30);
const READ: Duration = Duration::from_secs(10);
const PROOF: Duration = Duration::from_secs(90);
const RECORDS: usize = 256;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Creating,
    Created,
    Starting,
    Running,
    Updating,
    Updated,
    Stopping,
    Stopped,
    Removing,
    Removed,
}
#[derive(Clone, PartialEq)]
struct Target {
    context: wire::ComputerContext,
    resource: wire::ManagedResource,
    saved: wire::ManagedResource,
    initial: bool,
    never_started: bool,
    population: Option<bool>,
    image: Option<String>,
}
impl fmt::Debug for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NativeTarget([REDACTED])")
    }
}
impl Target {
    fn same_identity(&self, other: &Self) -> bool {
        let normalize = |mut value: Self| {
            value.context.request_id.clear();
            value.resource.absence_confirmed = false;
            value.saved.absence_confirmed = false;
            value.never_started = false;
            value.population = None;
            value.image = None;
            value
        };
        normalize(self.clone()) == normalize(other.clone())
    }
    fn id(&self) -> Result<&str, Error> {
        match &self.resource.native {
            Some(Native::DockerContainer(value)) => {
                value.container_id.as_deref().ok_or(Error::Uncertain)
            }
            _ => Err(Error::Ownership),
        }
    }
}
#[derive(Debug, Clone)]
struct Record {
    target: Target,
    stage: Stage,
}
#[derive(Debug, Default)]
pub(crate) struct Progress(Mutex<BTreeMap<String, Record>>);

macro_rules! token {
    ($($name:ident),+) => { $(#[derive(Debug)] pub struct $name(Target);)+ };
}
token!(
    PreparedContainer,
    Retirement,
    UpdatedContainer,
    StoppedContainer
);
impl PreparedContainer {
    pub fn resource(&self) -> &wire::ManagedResource {
        &self.0.resource
    }
}

fn state_time(value: Option<&str>) -> Option<jiff::Timestamp> {
    value
        .and_then(|value| value.parse::<jiff::Timestamp>().ok())
        .filter(|value| *value > jiff::Timestamp::UNIX_EPOCH)
}
fn flags(state: &ContainerState) -> bool {
    state.paused == Some(false) && state.restarting == Some(false) && state.dead == Some(false)
}
fn no_timestamp(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        value
            .parse::<jiff::Timestamp>()
            .is_ok_and(|value| value <= jiff::Timestamp::UNIX_EPOCH)
    })
}
fn running(state: &ContainerState) -> bool {
    state.status == Some(Status::RUNNING)
        && state.running == Some(true)
        && flags(state)
        && state.pid.is_some_and(|pid| pid > 0)
        && state_time(state.started_at.as_deref()).is_some_and(|started| {
            no_timestamp(state.finished_at.as_deref())
                || state_time(state.finished_at.as_deref())
                    .is_some_and(|finished| finished < started)
        })
}
fn stopped(state: &ContainerState, never_started: bool) -> bool {
    if state.running != Some(false) || !flags(state) || state.pid != Some(0) {
        return false;
    }
    if never_started && state.status == Some(Status::CREATED) {
        return no_timestamp(state.started_at.as_deref())
            && no_timestamp(state.finished_at.as_deref());
    }
    state.status == Some(Status::EXITED)
        && state_time(state.finished_at.as_deref()).is_some_and(|finished| {
            state_time(state.started_at.as_deref()).is_some_and(|started| finished >= started)
        })
}
fn restarting(state: &ContainerState) -> bool {
    state.status == Some(Status::RESTARTING)
        && state.running == Some(true)
        && state.restarting == Some(true)
        && state.paused == Some(false)
        && state.dead == Some(false)
        && state.pid == Some(0)
        && state_time(state.finished_at.as_deref()).is_some_and(|finished| {
            state_time(state.started_at.as_deref()).is_some_and(|started| finished >= started)
        })
}
impl Docker {
    /// Only local correlation stage, without native names or payloads.
    pub fn container_progress_diagnostic(&self, execution: &str) -> Option<&'static str> {
        let records = self.progress.0.lock().ok()?;
        records.get(execution).map(|record| match record.stage {
            Stage::Creating => "creating",
            Stage::Created => "created",
            Stage::Starting => "starting",
            Stage::Running => "running",
            Stage::Updating => "updating",
            Stage::Updated => "updated",
            Stage::Stopping => "stopping",
            Stage::Stopped => "stopped",
            Stage::Removing => "removing",
            Stage::Removed => "removed",
        })
    }
    pub async fn observe_container_absence(
        &self,
        context: &wire::ComputerContext,
        resource: &wire::ManagedResource,
    ) -> Result<Observation, Error> {
        let target = self.container_target(context, resource)?;
        self.verify_daemon().await?;
        let id = target.container_id.as_deref().unwrap_or(&resource.name);
        let result = tokio::time::timeout(READ, self.sdk.inspect_container(id, None))
            .await
            .map_err(|_| Error::Transport)?;
        let presence = match result {
            Err(error) if not_found(&error) && resource.absence_confirmed => Presence::Absent,
            Err(error) if not_found(&error) => Presence::Unknown,
            Ok(_) if resource.absence_confirmed => return Err(Error::Ownership),
            Ok(_) => Presence::Unknown,
            Err(error) => return Err(error.into()),
        };
        Ok(Observation {
            resource: resource.clone(),
            presence,
            running: false,
            terminal: false,
        })
    }
    fn container_target<'a>(
        &self,
        context: &wire::ComputerContext,
        value: &'a wire::ManagedResource,
    ) -> Result<&'a wire::DockerContainerNative, Error> {
        self.labels(context, value)?;
        let Some(Native::DockerContainer(native)) = &value.native else {
            return Err(Error::Ownership);
        };
        if value.kind != wire::ResourceKind::DockerContainer as i32
            || !value.namespace.is_empty()
            || value.uid.is_some()
            || value.execution_id.as_deref() != Some(&context.execution_id)
            || context.execution_id.is_empty()
            || native.daemon_id != self.daemon.as_str()
        {
            return Err(Error::Ownership);
        }
        NativeReservation::new(
            ManagedResourceKind::DockerContainer,
            NativeScope::DockerDaemon(self.daemon.clone()),
            value.name.clone(),
        )
        .map_err(|_| Error::Ownership)?;
        if let Some(id) = &native.container_id {
            DockerContainerId::parse(id.clone()).map_err(|_| Error::Ownership)?;
        }
        Ok(native)
    }
    fn stage(&self, target: &Target) -> Result<Option<Stage>, Error> {
        let records = self.progress.0.lock().map_err(|_| Error::Uncertain)?;
        match records.get(&target.context.execution_id) {
            Some(record) if record.target.same_identity(target) => Ok(Some(record.stage)),
            Some(_) => Err(Error::Ownership),
            None => Ok(None),
        }
    }
    fn record(&self, target: Target, expected: Option<Stage>, stage: Stage) -> Result<(), Error> {
        let mut records = self.progress.0.lock().map_err(|_| Error::Uncertain)?;
        let existing = records.get(&target.context.execution_id);
        if existing.map(|record| record.stage) != expected
            || existing.is_some_and(|record| !record.target.same_identity(&target))
        {
            return Err(Error::Uncertain);
        }
        if existing.is_none() && records.len() >= RECORDS {
            let remove = records
                .iter()
                .find(|(_, record)| record.stage == Stage::Removed)
                .map(|(key, _)| key.clone())
                .ok_or(if stage == Stage::Creating {
                    Error::ContainerCapacity
                } else {
                    Error::Uncertain
                })?;
            records.remove(&remove);
        }
        records.insert(
            target.context.execution_id.clone(),
            Record { target, stage },
        );
        Ok(())
    }
    async fn inspect_target(
        &self,
        target: &Target,
        bootstrap: Option<&Bootstrap<'_>>,
        retiring: bool,
    ) -> Result<ContainerInspectResponse, Error> {
        self.container_target(&target.context, &target.resource)?;
        self.verify_daemon().await?;
        let observed = tokio::time::timeout(READ, self.sdk.inspect_container(target.id()?, None))
            .await
            .map_err(|_| Error::Transport)??;
        if observed.id.as_deref() != Some(target.id()?) {
            return Err(Error::Ownership);
        }
        self.validate_container_profile(target, &observed, bootstrap, retiring)?;
        Ok(observed)
    }
    async fn original_volume(&self, target: &Target) -> Result<(), Error> {
        if self.volume_target(&target.saved)?.identity.is_none()
            || self
                .observe_volume(&target.context, &target.saved)
                .await?
                .presence
                != Presence::Present
        {
            return Err(Error::MissingEnvironment);
        }
        Ok(())
    }

    /// Exactly one fresh Prepare owns this call. An error never creates permission
    /// to call it again; durable preparation admission is owned by Control.
    pub async fn prepare_container(
        &self,
        context: &wire::ComputerContext,
        resource: &wire::ManagedResource,
        prepared_volume: &crate::PreparedVolume,
        image: &crate::PreparedImage,
        bootstrap: &Bootstrap<'_>,
        initial: bool,
    ) -> Result<PreparedContainer, Error> {
        let populate = prepared_volume.populate;
        let saved = prepared_volume.resource();
        if (populate && !initial)
            || image.configured != self.config.image
            || image.daemon != self.daemon
        {
            return Err(Error::Ownership);
        }
        let native = self.container_target(context, resource)?;
        if resource.absence_confirmed {
            return Err(Error::Ownership);
        }
        let mut profile = self.container_profile(context, resource, saved, bootstrap, populate)?;
        // Create by the validated immutable digest, not a tag which can change
        // between image preparation and native submission.
        profile.image = Some(image.digest.clone());
        let mut target = Target {
            context: context.clone(),
            resource: resource.clone(),
            saved: saved.clone(),
            initial,
            never_started: true,
            population: Some(populate),
            image: Some(image.digest.clone()),
        };
        self.original_volume(&target).await?;
        self.verify_daemon().await?;
        let lookup = native.container_id.as_deref().unwrap_or(&resource.name);
        let observed = tokio::time::timeout(READ, self.sdk.inspect_container(lookup, None))
            .await
            .map_err(|_| Error::Transport)?;
        let value = match observed {
            Ok(value) => value,
            Err(error) if not_found(&error) && native.container_id.is_none() => {
                self.record(target.clone(), None, Stage::Creating)?;
                let sdk = self.sdk.clone().with_timeout(REQUEST);
                let created = tokio::time::timeout(
                    REQUEST,
                    sdk.create_container(
                        Some(
                            CreateContainerOptionsBuilder::default()
                                .name(&resource.name)
                                .build(),
                        ),
                        profile,
                    ),
                )
                .await
                .map_err(|_| Error::Uncertain)?;
                let created = match created {
                    Ok(created) => created,
                    Err(error) if crate::volumes::create_rejected(&error) => {
                        let mut records = self.progress.0.lock().map_err(|_| Error::Uncertain)?;
                        if records.get(&context.execution_id).is_some_and(|record| {
                            record.stage == Stage::Creating && record.target == target
                        }) {
                            records.remove(&context.execution_id);
                        }
                        return Err(Error::CreationRejected(error));
                    }
                    Err(error) => return Err(error.into()),
                };
                DockerContainerId::parse(created.id.clone()).map_err(|_| Error::Ownership)?;
                // Change the candidate record only after its create positively
                // settled. No cache removal occurs on a lost native response.
                {
                    let mut records = self.progress.0.lock().map_err(|_| Error::Uncertain)?;
                    let record = records
                        .get_mut(&context.execution_id)
                        .ok_or(Error::Uncertain)?;
                    if record.stage != Stage::Creating || record.target != target {
                        return Err(Error::Uncertain);
                    }
                    if let Some(Native::DockerContainer(value)) = target.resource.native.as_mut() {
                        value.container_id = Some(created.id);
                    }
                    record.target = target.clone();
                    record.stage = Stage::Created;
                }
                return Ok(PreparedContainer({
                    self.inspect_target(&target, Some(bootstrap), false).await?;
                    target
                }));
            }
            Err(error) => return Err(error.into()),
        };
        let id = value.id.clone().ok_or(Error::Ownership)?;
        DockerContainerId::parse(id.clone()).map_err(|_| Error::Ownership)?;
        if native
            .container_id
            .as_ref()
            .is_some_and(|expected| *expected != id)
        {
            return Err(Error::Ownership);
        }
        if let Some(Native::DockerContainer(value)) = target.resource.native.as_mut() {
            value.container_id = Some(id);
        }
        self.validate_container_profile(&target, &value, Some(bootstrap), false)?;
        // Existing created objects can be recovered only within the fresh Prepare
        // boundary and positively never-started exact native state.
        if !value
            .state
            .as_ref()
            .is_some_and(|value| stopped(value, true))
        {
            return Err(Error::Uncertain);
        }
        match self.stage(&target)? {
            None => self.record(target.clone(), None, Stage::Created)?,
            Some(Stage::Created) => {}
            _ => return Err(Error::Uncertain),
        }
        Ok(PreparedContainer(target))
    }

    /// Allocation must have been ACKed and the current command rechecked by the
    /// caller. The consumed token plus stage CAS prevents another Start.
    pub async fn start_container(&self, prepared: PreparedContainer) -> Result<Observation, Error> {
        let mut target = prepared.0;
        self.original_volume(&target).await?;
        let observed = self.inspect_target(&target, None, false).await?;
        if !observed
            .state
            .as_ref()
            .is_some_and(|state| stopped(state, target.never_started))
        {
            return Err(Error::Uncertain);
        }
        target.never_started = false;
        self.record(target.clone(), Some(Stage::Created), Stage::Starting)?;
        let sdk = self.sdk.clone().with_timeout(REQUEST);
        tokio::time::timeout(REQUEST, sdk.start_container(target.id()?, None))
            .await
            .map_err(|_| Error::Uncertain)??;
        self.record(target.clone(), Some(Stage::Starting), Stage::Running)?;
        self.observe_container(
            &target.context,
            &target.resource,
            &target.saved,
            target.initial,
            true,
        )
        .await
    }

    pub async fn observe_container(
        &self,
        context: &wire::ComputerContext,
        resource: &wire::ManagedResource,
        saved: &wire::ManagedResource,
        initial: bool,
        prior_running: bool,
    ) -> Result<Observation, Error> {
        let native = self.container_target(context, resource)?;
        let target = Target {
            context: context.clone(),
            resource: resource.clone(),
            saved: saved.clone(),
            initial,
            never_started: false,
            population: None,
            image: None,
        };
        if native.container_id.is_none() {
            self.verify_daemon().await?;
            let observed =
                tokio::time::timeout(READ, self.sdk.inspect_container(&resource.name, None))
                    .await
                    .map_err(|_| Error::Transport)?;
            return match observed {
                Err(error) if not_found(&error) => Ok(Observation {
                    resource: resource.clone(),
                    presence: if resource.absence_confirmed {
                        Presence::Absent
                    } else {
                        Presence::Unknown
                    },
                    running: false,
                    terminal: false,
                }),
                Ok(_) if resource.absence_confirmed => Err(Error::Ownership),
                Ok(_) => Ok(Observation {
                    resource: resource.clone(),
                    presence: Presence::Unknown,
                    running: false,
                    terminal: false,
                }),
                Err(error) => Err(error.into()),
            };
        }
        let stage = self.stage(&target)?;
        let retiring = matches!(
            stage,
            Some(
                Stage::Updating
                    | Stage::Updated
                    | Stage::Stopping
                    | Stage::Stopped
                    | Stage::Removing
                    | Stage::Removed
            )
        );
        match self.inspect_target(&target, None, retiring).await {
            Ok(value) => {
                if resource.absence_confirmed {
                    return Err(Error::Ownership);
                }
                let settled =
                    matches!(stage, Some(Stage::Running)) || (stage.is_none() && prior_running);
                let running = settled && value.state.as_ref().is_some_and(running);
                if running {
                    self.original_volume(&target).await?;
                }
                Ok(Observation {
                    resource: resource.clone(),
                    presence: Presence::Present,
                    running,
                    terminal: false,
                })
            }
            Err(Error::Engine(error)) if not_found(&error) => Ok(Observation {
                resource: resource.clone(),
                presence: if resource.absence_confirmed || stage == Some(Stage::Removed) {
                    Presence::Absent
                } else {
                    Presence::Unknown
                },
                running: false,
                terminal: false,
            }),
            Err(error) => Err(error),
        }
    }

    /// Begin only for the server's exact live Retire command. An older persisted
    /// Running completion cannot override a same-process unknown native stage.
    pub async fn begin_retirement(
        &self,
        context: &wire::ComputerContext,
        resource: &wire::ManagedResource,
        saved: &wire::ManagedResource,
        initial: bool,
        prior_running: bool,
        require_terminal: bool,
    ) -> Result<Retirement, Error> {
        self.container_target(context, resource)?;
        let mut target = Target {
            context: context.clone(),
            resource: resource.clone(),
            saved: saved.clone(),
            initial,
            never_started: false,
            population: None,
            image: None,
        };
        self.original_volume(&target).await?;
        let stage = self.stage(&target)?;
        target.never_started = stage == Some(Stage::Created);
        if !matches!(stage, Some(Stage::Created | Stage::Running))
            && !(stage.is_none() && prior_running)
        {
            return Err(Error::Uncertain);
        }
        let observed = self.inspect_target(&target, None, false).await?;
        let state = observed.state.as_ref().ok_or(Error::Uncertain)?;
        if require_terminal {
            return Err(Error::Uncertain);
        } // Active `always` never proves terminal retirement.
        if !(running(state) || restarting(state) || stopped(state, stage == Some(Stage::Created))) {
            return Err(Error::Uncertain);
        }
        if stage.is_none() {
            self.record(target.clone(), None, Stage::Running)?;
        }
        Ok(Retirement(target))
    }
    pub async fn disable_restart(&self, retirement: Retirement) -> Result<UpdatedContainer, Error> {
        let target = retirement.0;
        let stage = self.stage(&target)?.ok_or(Error::Uncertain)?;
        if !matches!(stage, Stage::Created | Stage::Running) {
            return Err(Error::Uncertain);
        }
        self.inspect_target(&target, None, false).await?;
        self.record(target.clone(), Some(stage), Stage::Updating)?;
        let sdk = self.sdk.clone().with_timeout(REQUEST);
        tokio::time::timeout(
            REQUEST,
            sdk.update_container(
                target.id()?,
                ContainerUpdateBody {
                    restart_policy: Some(RestartPolicy {
                        name: Some(RestartPolicyNameEnum::NO),
                        maximum_retry_count: Some(0),
                    }),
                    ..Default::default()
                },
            ),
        )
        .await
        .map_err(|_| Error::Uncertain)??;
        let value = self.inspect_target(&target, None, true).await?;
        if value
            .host_config
            .as_ref()
            .and_then(|host| host.restart_policy.as_ref())
            .and_then(|policy| policy.name)
            != Some(RestartPolicyNameEnum::NO)
        {
            return Err(Error::Uncertain);
        }
        self.record(target.clone(), Some(Stage::Updating), Stage::Updated)?;
        Ok(UpdatedContainer(target))
    }
    pub async fn stop_container(
        &self,
        updated: UpdatedContainer,
    ) -> Result<StoppedContainer, Error> {
        let target = updated.0;
        let observed = self.inspect_target(&target, None, true).await?;
        let state = observed.state.as_ref().ok_or(Error::Uncertain)?;
        self.record(target.clone(), Some(Stage::Updated), Stage::Stopping)?;
        if !stopped(state, target.never_started) {
            if !(running(state) || restarting(state)) {
                return Err(Error::Uncertain);
            }
            let budget = Duration::from_secs(45);
            let sdk = self.sdk.clone().with_timeout(budget);
            tokio::time::timeout(
                budget,
                sdk.stop_container(
                    target.id()?,
                    Some(
                        StopContainerOptionsBuilder::default()
                            .t(30)
                            .signal("SIGTERM")
                            .build(),
                    ),
                ),
            )
            .await
            .map_err(|_| Error::Uncertain)??;
        }
        let value = self.inspect_target(&target, None, true).await?;
        if !value
            .state
            .as_ref()
            .is_some_and(|state| stopped(state, target.never_started))
        {
            return Err(Error::Uncertain);
        }
        self.record(target.clone(), Some(Stage::Stopping), Stage::Stopped)?;
        Ok(StoppedContainer(target))
    }
    pub async fn remove_container(
        &self,
        stopped_token: StoppedContainer,
    ) -> Result<Observation, Error> {
        let target = stopped_token.0;
        let observed = self.inspect_target(&target, None, true).await?;
        if !observed
            .state
            .as_ref()
            .is_some_and(|state| stopped(state, target.never_started))
        {
            return Err(Error::Uncertain);
        }
        self.record(target.clone(), Some(Stage::Stopped), Stage::Removing)?;
        let sdk = self.sdk.clone().with_timeout(REQUEST);
        tokio::time::timeout(
            REQUEST,
            sdk.remove_container(
                target.id()?,
                Some(
                    RemoveContainerOptionsBuilder::default()
                        .force(false)
                        .v(false)
                        .build(),
                ),
            ),
        )
        .await
        .map_err(|_| Error::Uncertain)??;
        let deadline = tokio::time::Instant::now() + PROOF;
        loop {
            self.verify_daemon().await?;
            let observed =
                tokio::time::timeout_at(deadline, self.sdk.inspect_container(target.id()?, None))
                    .await
                    .map_err(|_| Error::Uncertain)?;
            match observed {
                Err(error) if not_found(&error) => {
                    self.record(target.clone(), Some(Stage::Removing), Stage::Removed)?;
                    return Ok(Observation {
                        resource: target.resource,
                        presence: Presence::Absent,
                        running: false,
                        terminal: false,
                    });
                }
                Err(error) => return Err(error.into()),
                Ok(value) if value.id.as_deref() != Some(target.id()?) => {
                    return Err(Error::Ownership);
                }
                Ok(_) => {}
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(Error::Uncertain);
            }
            tokio::time::sleep_until(
                (tokio::time::Instant::now() + Duration::from_millis(250)).min(deadline),
            )
            .await;
        }
    }
}
