//! Original local-volume identity and individually scoped effects. The caller
//! owns exact Control admission/check/report ACKs between native stages.
use crate::{Docker, Error};
use bollard::{
    errors::Error as EngineError,
    models::{Volume, VolumeCreateRequest},
    query_parameters::RemoveVolumeOptionsBuilder,
};
use chapchoo_domain::{DockerVolumeIdentity, ManagedResourceKind, NativeReservation, NativeScope};
use chapchoo_protocol::control::{self as wire, managed_resource::Native};
use std::{
    collections::{BTreeMap, HashMap},
    time::Duration,
};

const PROVIDER: &str = "chapchoo.io/provider";
const COMPUTER: &str = "chapchoo.io/computer";
pub(crate) const EXECUTION: &str = "chapchoo.io/execution";
const NATIVE_BUDGET: Duration = Duration::from_secs(30);
const PROOF_BUDGET: Duration = Duration::from_secs(90);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    Present,
    Absent,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub resource: wire::ManagedResource,
    pub presence: Presence,
    pub running: bool,
    pub terminal: bool,
}
/// Only a positively completed fresh volume creation may populate image data.
/// Recovery and every existing original volume use no-copy, even when the
/// execution's initial-allocation reservation fact is still true.
#[derive(Debug)]
pub struct PreparedVolume {
    resource: wire::ManagedResource,
    pub(crate) populate: bool,
}
impl PreparedVolume {
    pub fn resource(&self) -> &wire::ManagedResource {
        &self.resource
    }
}

pub(crate) fn not_found(error: &EngineError) -> bool {
    matches!(
        error,
        EngineError::DockerResponseServerError {
            status_code: 404,
            ..
        }
    )
}
pub(crate) fn create_rejected(error: &EngineError) -> bool {
    matches!(
        error,
        EngineError::DockerResponseServerError {
            status_code: 400 | 401 | 403 | 404 | 405 | 422 | 429,
            ..
        }
    )
}
pub(crate) fn volume_identity(
    value: &wire::DockerVolumeIdentity,
) -> Result<DockerVolumeIdentity, Error> {
    DockerVolumeIdentity::local(
        &value.driver,
        &value.scope,
        &value
            .options
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        value.created_at.clone(),
        value
            .labels
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )
    .map_err(|_| Error::Ownership)
}
fn encoded_identity(value: &DockerVolumeIdentity) -> wire::DockerVolumeIdentity {
    wire::DockerVolumeIdentity {
        driver: "local".into(),
        scope: "local".into(),
        options: Default::default(),
        created_at: value.created_at().map(str::to_owned),
        labels: value
            .labels()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    }
}

impl Docker {
    pub(crate) fn labels(
        &self,
        context: &wire::ComputerContext,
        resource: &wire::ManagedResource,
    ) -> Result<HashMap<String, String>, Error> {
        let source = resource.allocation.as_ref().ok_or(Error::Ownership)?;
        let source = chapchoo_domain::NativeAllocationWitness::new(
            source.computer_id.clone(),
            source.provider_id.clone(),
            source.execution_id.clone(),
        )
        .map_err(|_| Error::Ownership)?;
        if context.computer_id.is_empty()
            || context.request_id.is_empty()
            || source.execution().is_some() != resource.execution_id.is_some()
            || resource
                .execution_id
                .as_ref()
                .is_some_and(|execution| execution != &context.execution_id || execution.is_empty())
        {
            return Err(Error::Ownership);
        }
        let mut labels = HashMap::from([
            (PROVIDER.into(), source.provider().to_owned()),
            (COMPUTER.into(), source.computer().to_owned()),
        ]);
        if let Some(execution) = source.execution() {
            labels.insert(EXECUTION.into(), execution.to_owned());
        }
        Ok(labels)
    }

    pub(crate) fn volume_target<'a>(
        &self,
        resource: &'a wire::ManagedResource,
    ) -> Result<&'a wire::DockerVolumeNative, Error> {
        let Some(Native::DockerVolume(native)) = &resource.native else {
            return Err(Error::Ownership);
        };
        if resource.kind != wire::ResourceKind::DockerVolume as i32
            || !resource.namespace.is_empty()
            || resource.uid.is_some()
            || resource.execution_id.is_some()
            || native.daemon_id != self.daemon.as_str()
        {
            return Err(Error::Ownership);
        }
        NativeReservation::new(
            ManagedResourceKind::DockerVolume,
            NativeScope::DockerDaemon(self.daemon.clone()),
            resource.name.clone(),
        )
        .map_err(|_| Error::Ownership)?;
        if let Some(identity) = &native.identity {
            volume_identity(identity)?;
        }
        Ok(native)
    }

    fn allocated_volume(
        &self,
        context: &wire::ComputerContext,
        reserved: &wire::ManagedResource,
        volume: &Volume,
    ) -> Result<wire::ManagedResource, Error> {
        let native = self.volume_target(reserved)?;
        if reserved.absence_confirmed {
            return Err(if native.identity.is_some() {
                Error::MissingEnvironment
            } else {
                Error::Ownership
            });
        }
        if volume.name != reserved.name || volume.labels != self.labels(context, reserved)? {
            return Err(Error::Ownership);
        }
        let identity = DockerVolumeIdentity::local(
            &volume.driver,
            volume.scope.as_ref().map(AsRef::as_ref).unwrap_or(""),
            &volume
                .options
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<BTreeMap<_, _>>(),
            volume.created_at.clone(),
            volume
                .labels
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        )
        .map_err(|_| Error::Ownership)?;
        if let Some(expected) = &native.identity
            && volume_identity(expected)? != identity
        {
            return Err(Error::MissingEnvironment);
        }
        let mut resource = reserved.clone();
        resource.absence_confirmed = false;
        resource.native = Some(Native::DockerVolume(wire::DockerVolumeNative {
            daemon_id: self.daemon.as_str().into(),
            identity: Some(encoded_identity(&identity)),
        }));
        Ok(resource)
    }

    pub async fn observe_volume(
        &self,
        context: &wire::ComputerContext,
        resource: &wire::ManagedResource,
    ) -> Result<Observation, Error> {
        let target = self.volume_target(resource)?;
        self.labels(context, resource)?;
        self.verify_daemon().await?;
        match tokio::time::timeout(
            Duration::from_secs(10),
            self.sdk.inspect_volume(&resource.name),
        )
        .await
        .map_err(|_| Error::Transport)?
        {
            Ok(value) => Ok(Observation {
                resource: self.allocated_volume(context, resource, &value)?,
                presence: Presence::Present,
                running: false,
                terminal: false,
            }),
            Err(error) if not_found(&error) => Ok(Observation {
                resource: resource.clone(),
                presence: if target.identity.is_some() || resource.absence_confirmed {
                    Presence::Absent
                } else {
                    Presence::Unknown
                },
                running: false,
                terminal: false,
            }),
            Err(error) => Err(error.into()),
        }
    }

    /// Called once under a freshly admitted Prepare. An allocated original is
    /// inspected only; even its first execution cannot replace it after loss.
    pub async fn ensure_volume(
        &self,
        context: &wire::ComputerContext,
        resource: &wire::ManagedResource,
        initial_allocation: bool,
    ) -> Result<PreparedVolume, Error> {
        let target = self.volume_target(resource)?;
        let labels = self.labels(context, resource)?;
        self.verify_daemon().await?;
        match tokio::time::timeout(
            Duration::from_secs(10),
            self.sdk.inspect_volume(&resource.name),
        )
        .await
        .map_err(|_| Error::Transport)?
        {
            Ok(value) => {
                return Ok(PreparedVolume {
                    resource: self.allocated_volume(context, resource, &value)?,
                    populate: false,
                });
            }
            Err(error) if not_found(&error) => {
                if target.identity.is_some() || !initial_allocation || resource.absence_confirmed {
                    return Err(Error::MissingEnvironment);
                }
            }
            Err(error) => return Err(error.into()),
        }
        let sdk = self.sdk.clone().with_timeout(NATIVE_BUDGET);
        let created = tokio::time::timeout(
            NATIVE_BUDGET,
            sdk.create_volume(VolumeCreateRequest {
                name: Some(resource.name.clone()),
                driver: Some("local".into()),
                driver_opts: Some(HashMap::new()),
                labels: Some(labels),
                cluster_volume_spec: None,
            }),
        )
        .await
        .map_err(|_| Error::Uncertain)?;
        let created = match created {
            Ok(created) => created,
            Err(error) if create_rejected(&error) => return Err(Error::CreationRejected(error)),
            Err(error) => return Err(error.into()),
        };
        Ok(PreparedVolume {
            resource: self.allocated_volume(context, resource, &created)?,
            populate: true,
        })
    }

    /// The caller checks the exact M16 never-submitted suffix before this stage.
    /// Local volumes have no UID-conditional removal; exclusive ownership of the
    /// reserved name is an explicit trusted-daemon boundary.
    pub async fn remove_volume(
        &self,
        context: &wire::ComputerContext,
        resource: &wire::ManagedResource,
    ) -> Result<Observation, Error> {
        if self.volume_target(resource)?.identity.is_none() {
            let observed = self.observe_volume(context, resource).await?;
            return if resource.absence_confirmed && observed.presence == Presence::Absent {
                Ok(observed)
            } else {
                Err(Error::Uncertain)
            };
        }
        let observed = self.observe_volume(context, resource).await?;
        if observed.presence == Presence::Absent {
            return Ok(observed);
        }
        if observed.presence != Presence::Present {
            return Err(Error::Uncertain);
        }
        self.verify_daemon().await?;
        let sdk = self.sdk.clone().with_timeout(NATIVE_BUDGET);
        tokio::time::timeout(
            NATIVE_BUDGET,
            sdk.remove_volume(
                &resource.name,
                Some(RemoveVolumeOptionsBuilder::default().force(false).build()),
            ),
        )
        .await
        .map_err(|_| Error::Uncertain)??;
        let deadline = tokio::time::Instant::now() + PROOF_BUDGET;
        loop {
            let observed =
                tokio::time::timeout_at(deadline, self.observe_volume(context, resource))
                    .await
                    .map_err(|_| Error::Uncertain)??;
            if observed.presence == Presence::Absent {
                return Ok(observed);
            }
            tokio::time::sleep_until(
                (tokio::time::Instant::now() + Duration::from_millis(250)).min(deadline),
            )
            .await;
        }
    }
}
