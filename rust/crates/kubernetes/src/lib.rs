//! Narrow standalone Pod/PVC/bootstrap-Secret adapter over the supported client.
//! The caller owns source admission, SDK task lifetime and durable observations.
use chapchoo_protocol::control::{self as wire, ResourceKind};
use futures_util::{StreamExt, TryStreamExt};
use k8s_openapi::{
    ByteString,
    api::core::v1::{
        Capabilities, Container, EnvVar, EnvVarSource, PersistentVolumeClaim,
        PersistentVolumeClaimSpec, PersistentVolumeClaimVolumeSource, Pod, PodSecurityContext,
        PodSpec, ResourceRequirements, SeccompProfile, Secret, SecretKeySelector, SecurityContext,
        Volume, VolumeMount, VolumeResourceRequirements,
    },
    apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta},
};
use kube::{
    Api, Client,
    api::{DeleteParams, PostParams, Preconditions, WatchEvent, WatchParams},
};
use std::{collections::BTreeMap, fmt};

pub mod authentication;

const PROVIDER: &str = "chapchoo.io/provider";
const COMPUTER: &str = "chapchoo.io/computer";
const EXECUTION: &str = "chapchoo.io/execution";

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Kubernetes operation failed; its external outcome may be uncertain")]
    Client(#[from] kube::Error),
    #[error("Kubernetes explicitly rejected resource creation")]
    CreationRejected(#[source] kube::Error),
    #[error("Managed resource identity or ownership does not match")]
    Ownership,
    #[error("The original saved environment is missing or unverified")]
    MissingEnvironment,
    #[error("Managed Kubernetes configuration is invalid")]
    Configuration,
    #[error("Managed resource deletion has not reached confirmed absence")]
    DeletionUncertain,
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("KubernetesError([REDACTED])")
    }
}
impl Error {
    /// An explicit native POST rejection proves create non-acceptance.
    /// GET, collision follow-up, transport and server errors remain uncertain.
    pub fn definitely_rejected(&self) -> bool {
        matches!(self, Self::CreationRejected(_))
    }
}
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
#[derive(Clone)]
pub struct Config {
    pub namespace: String,
    pub image: String,
    pub image_pull_policy: String,
    pub storage_class: Option<String>,
    pub storage_request: String,
    pub saved_mount: String,
    pub service_account: String,
    pub cpu_request: String,
    pub memory_request: String,
    pub cpu_limit: String,
    pub memory_limit: String,
}
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("KubernetesConfig([REDACTED])")
    }
}
#[derive(Clone)]
pub struct Kubernetes {
    client: Client,
    config: Config,
}
impl fmt::Debug for Kubernetes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Kubernetes([REDACTED])")
    }
}
impl Kubernetes {
    pub fn namespace(&self) -> &str {
        &self.config.namespace
    }
    pub fn in_cluster(config: Config) -> Result<Self, Error> {
        let client = kube::Config::incluster().map_err(|_| Error::Configuration)?;
        Self::from_client_config(client, config)
    }
    pub async fn from_kubeconfig(path: &std::path::Path, config: Config) -> Result<Self, Error> {
        let file = kube::config::Kubeconfig::read_from(path).map_err(|_| Error::Configuration)?;
        let client =
            kube::Config::from_custom_kubeconfig(file, &kube::config::KubeConfigOptions::default())
                .await
                .map_err(|_| Error::Configuration)?;
        Self::from_client_config(client, config)
    }
    /// Caller selects explicit in-cluster/kubeconfig credentials. Disable the
    /// SDK's default server retry layer before building any effect-capable client.
    pub fn from_client_config(mut client: kube::Config, config: Config) -> Result<Self, Error> {
        client.default_retry = false;
        client.connect_timeout = Some(std::time::Duration::from_secs(5));
        client.read_timeout = Some(std::time::Duration::from_secs(30));
        client.write_timeout = Some(std::time::Duration::from_secs(30));
        Self::new(Client::try_from(client)?, config)
    }
    pub fn new(client: Client, config: Config) -> Result<Self, Error> {
        if config.namespace.is_empty()
            || config.image.is_empty()
            || config.storage_request.is_empty()
            || !config.saved_mount.starts_with('/')
            || config.service_account.is_empty()
            || [
                &config.cpu_request,
                &config.memory_request,
                &config.cpu_limit,
                &config.memory_limit,
            ]
            .iter()
            .any(|v| v.is_empty())
            || !matches!(
                config.image_pull_policy.as_str(),
                "Always" | "IfNotPresent" | "Never"
            )
        {
            return Err(Error::Configuration);
        }
        Ok(Self { client, config })
    }
    fn annotations(
        &self,
        resource: &wire::ManagedResource,
    ) -> Result<BTreeMap<String, String>, Error> {
        let source = resource.allocation.as_ref().ok_or(Error::Ownership)?;
        let source = chapchoo_domain::NativeAllocationWitness::new(
            source.computer_id.clone(),
            source.provider_id.clone(),
            source.execution_id.clone(),
        )
        .map_err(|_| Error::Ownership)?;
        if source.execution().is_some() != resource.execution_id.is_some() {
            return Err(Error::Ownership);
        }
        let mut annotations = BTreeMap::from([
            (PROVIDER.into(), source.provider().to_owned()),
            (COMPUTER.into(), source.computer().to_owned()),
        ]);
        if let Some(execution) = source.execution() {
            annotations.insert(EXECUTION.into(), execution.to_owned());
        }
        Ok(annotations)
    }
    fn metadata(
        &self,
        computer: &wire::ComputerContext,
        resource: &wire::ManagedResource,
    ) -> Result<ObjectMeta, Error> {
        let _ = self.target(computer, resource)?;
        Ok(ObjectMeta {
            name: Some(resource.name.clone()),
            namespace: Some(resource.namespace.clone()),
            annotations: Some(self.annotations(resource)?),
            ..Default::default()
        })
    }
    fn validate(
        &self,
        computer: &wire::ComputerContext,
        resource: &wire::ManagedResource,
        metadata: &ObjectMeta,
    ) -> Result<String, Error> {
        let _ = self.target(computer, resource)?;
        if resource.namespace != self.config.namespace
            || metadata.namespace.as_deref() != Some(&resource.namespace)
            || metadata.name.as_deref() != Some(&resource.name)
        {
            return Err(Error::Ownership);
        }
        let uid = metadata
            .uid
            .as_ref()
            .filter(|s| !s.is_empty())
            .ok_or(Error::Ownership)?;
        if resource
            .uid
            .as_ref()
            .is_some_and(|expected| expected != uid)
        {
            return Err(Error::Ownership);
        }
        let annotations = metadata.annotations.as_ref().ok_or(Error::Ownership)?;
        if self
            .annotations(resource)?
            .iter()
            .any(|(key, value)| annotations.get(key) != Some(value))
        {
            return Err(Error::Ownership);
        }
        if resource.execution_id.is_none() && annotations.contains_key(EXECUTION) {
            return Err(Error::Ownership);
        }
        if metadata
            .owner_references
            .as_ref()
            .is_some_and(|owners| !owners.is_empty())
        {
            return Err(Error::Ownership);
        }
        Ok(uid.clone())
    }
    fn target(
        &self,
        computer: &wire::ComputerContext,
        resource: &wire::ManagedResource,
    ) -> Result<ResourceKind, Error> {
        let _ = self.annotations(resource)?;
        if computer.computer_id.is_empty()
            || resource.native.is_some()
            || resource.namespace != self.config.namespace
            || resource.name.is_empty()
        {
            return Err(Error::Ownership);
        }
        let kind = ResourceKind::try_from(resource.kind).map_err(|_| Error::Ownership)?;
        match kind {
            ResourceKind::Pod | ResourceKind::BootstrapSecret
                if resource.execution_id.as_deref() == Some(&computer.execution_id)
                    && !computer.execution_id.is_empty() =>
            {
                Ok(kind)
            }
            ResourceKind::Pvc if resource.execution_id.is_none() => Ok(kind),
            _ => Err(Error::Ownership),
        }
    }
    pub async fn observe(
        &self,
        computer: &wire::ComputerContext,
        resource: &wire::ManagedResource,
    ) -> Result<Observation, Error> {
        let kind = self.target(computer, resource)?;
        let mut observation = Observation {
            resource: resource.clone(),
            presence: Presence::Unknown,
            running: false,
            terminal: false,
        };
        let metadata = match kind {
            ResourceKind::Pod => {
                let api: Api<Pod> = Api::namespaced(self.client.clone(), &resource.namespace);
                if let Some(pod) = api.get_opt(&resource.name).await? {
                    self.validate(computer, resource, &pod.metadata)?;
                    if let Some(status) = &pod.status {
                        observation.running = status.phase.as_deref() == Some("Running")
                            && pod.metadata.deletion_timestamp.is_none();
                        observation.terminal = terminal_pod(&pod);
                    }
                    Some(pod.metadata)
                } else {
                    observation.presence = Presence::Absent;
                    None
                }
            }
            ResourceKind::Pvc => {
                let api: Api<PersistentVolumeClaim> =
                    Api::namespaced(self.client.clone(), &resource.namespace);
                api.get_opt(&resource.name).await?.map(|r| r.metadata)
            }
            ResourceKind::BootstrapSecret => {
                let api: Api<Secret> = Api::namespaced(self.client.clone(), &resource.namespace);
                api.get_opt(&resource.name).await?.map(|r| r.metadata)
            }
            _ => return Err(Error::Ownership),
        };
        if let Some(metadata) = metadata {
            if resource.uid.is_none() && resource.absence_confirmed {
                return Err(Error::Ownership);
            }
            observation.resource.uid = Some(self.validate(computer, resource, &metadata)?);
            observation.presence = Presence::Present;
        } else if kind != ResourceKind::Pod
            && (resource.uid.is_some() || resource.absence_confirmed)
        {
            observation.presence = Presence::Absent;
        }
        Ok(observation)
    }
    /// Initial allocation alone can create a PVC. An allocated original UID is
    /// never replaced, even for an initial execution retry after disappearance.
    pub async fn ensure_pvc(
        &self,
        computer: &wire::ComputerContext,
        resource: &wire::ManagedResource,
        initial_allocation: bool,
    ) -> Result<wire::ManagedResource, Error> {
        if self.target(computer, resource)? != ResourceKind::Pvc {
            return Err(Error::Ownership);
        }
        let api: Api<PersistentVolumeClaim> =
            Api::namespaced(self.client.clone(), &resource.namespace);
        if let Some(existing) = api.get_opt(&resource.name).await? {
            return self.owned(computer, resource, &existing.metadata);
        }
        if !initial_allocation || resource.uid.is_some() {
            return Err(Error::MissingEnvironment);
        }
        let pvc = PersistentVolumeClaim {
            metadata: self.metadata(computer, resource)?,
            spec: Some(PersistentVolumeClaimSpec {
                access_modes: Some(vec!["ReadWriteOnce".into()]),
                storage_class_name: self.config.storage_class.clone(),
                resources: Some(VolumeResourceRequirements {
                    requests: Some(BTreeMap::from([(
                        "storage".into(),
                        Quantity(self.config.storage_request.clone()),
                    )])),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let created = match api.create(&PostParams::default(), &pvc).await {
            Ok(created) => created,
            Err(kube::Error::Api(error)) if error.code == 409 => api.get(&resource.name).await?,
            Err(kube::Error::Api(error)) if matches!(error.code, 400 | 401 | 403 | 422 | 429) => {
                return Err(Error::CreationRejected(kube::Error::Api(error)));
            }
            Err(error) => return Err(error.into()),
        };
        self.owned(computer, resource, &created.metadata)
    }
    fn owned(
        &self,
        computer: &wire::ComputerContext,
        resource: &wire::ManagedResource,
        metadata: &ObjectMeta,
    ) -> Result<wire::ManagedResource, Error> {
        let mut owned = resource.clone();
        owned.uid = Some(self.validate(computer, resource, metadata)?);
        Ok(owned)
    }
    pub async fn ensure_bootstrap(
        &self,
        computer: &wire::ComputerContext,
        resource: &wire::ManagedResource,
        session_id: &str,
        initial_key: &str,
        endpoint: &str,
    ) -> Result<wire::ManagedResource, Error> {
        if self.target(computer, resource)? != ResourceKind::BootstrapSecret {
            return Err(Error::Ownership);
        }
        let api: Api<Secret> = Api::namespaced(self.client.clone(), &resource.namespace);
        let data = BTreeMap::from([
            (
                "session-id".into(),
                ByteString(session_id.as_bytes().to_vec()),
            ),
            (
                "initial-key".into(),
                ByteString(initial_key.as_bytes().to_vec()),
            ),
            (
                "control-endpoint".into(),
                ByteString(endpoint.as_bytes().to_vec()),
            ),
        ]);
        let existing = match api.get_opt(&resource.name).await? {
            Some(existing) => existing,
            None => {
                if resource.uid.is_some() {
                    return Err(Error::Ownership);
                }
                let secret = Secret {
                    metadata: self.metadata(computer, resource)?,
                    data: Some(data.clone()),
                    immutable: Some(true),
                    type_: Some("Opaque".into()),
                    ..Default::default()
                };
                match api.create(&PostParams::default(), &secret).await {
                    Ok(created) => created,
                    Err(kube::Error::Api(error)) if error.code == 409 => {
                        api.get(&resource.name).await?
                    }
                    Err(kube::Error::Api(error))
                        if matches!(error.code, 400 | 401 | 403 | 422 | 429) =>
                    {
                        return Err(Error::CreationRejected(kube::Error::Api(error)));
                    }
                    Err(error) => return Err(error.into()),
                }
            }
        };
        if existing.data.as_ref() != Some(&data) || existing.immutable != Some(true) {
            return Err(Error::Ownership);
        }
        self.owned(computer, resource, &existing.metadata)
    }
    pub async fn ensure_pod(
        &self,
        computer: &wire::ComputerContext,
        pod: &wire::ManagedResource,
        pvc: &wire::ManagedResource,
        bootstrap: &wire::ManagedResource,
    ) -> Result<wire::ManagedResource, Error> {
        if self.target(computer, pod)? != ResourceKind::Pod
            || pvc.uid.is_none()
            || bootstrap.uid.is_none()
        {
            return Err(Error::Ownership);
        }
        if self.observe(computer, pvc).await?.presence != Presence::Present
            || self.observe(computer, bootstrap).await?.presence != Presence::Present
        {
            return Err(Error::MissingEnvironment);
        }
        let api: Api<Pod> = Api::namespaced(self.client.clone(), &pod.namespace);
        if let Some(existing) = api.get_opt(&pod.name).await? {
            self.check_pod(&existing, pvc, bootstrap)?;
            return self.owned(computer, pod, &existing.metadata);
        }
        if pod.uid.is_some() {
            return Err(Error::Ownership);
        }
        let environment = [
            ("CHAPCHOO_CONNECTOR_SESSION_ID", "session-id"),
            ("CHAPCHOO_CONNECTOR_INITIAL_KEY", "initial-key"),
            ("CHAPCHOO_CONTROL_ENDPOINT", "control-endpoint"),
        ]
        .into_iter()
        .map(|(name, key)| EnvVar {
            name: name.into(),
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: bootstrap.name.clone(),
                    key: key.into(),
                    optional: Some(false),
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
        .collect();
        let created = Pod {
            metadata: self.metadata(computer, pod)?,
            spec: Some(PodSpec {
                service_account_name: Some(self.config.service_account.clone()),
                automount_service_account_token: Some(false),
                restart_policy: Some("Always".into()),
                security_context: Some(PodSecurityContext {
                    run_as_non_root: Some(true),
                    run_as_user: Some(10001),
                    fs_group: Some(10001),
                    seccomp_profile: Some(SeccompProfile {
                        type_: "RuntimeDefault".into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                containers: vec![Container {
                    name: "computer-connector".into(),
                    image: Some(self.config.image.clone()),
                    image_pull_policy: Some(self.config.image_pull_policy.clone()),
                    command: Some(vec!["/opt/chapchoo/chapchoo".into()]),
                    args: Some(vec!["computer-connector".into()]),
                    security_context: Some(SecurityContext {
                        allow_privilege_escalation: Some(false),
                        read_only_root_filesystem: Some(true),
                        capabilities: Some(Capabilities {
                            drop: Some(vec!["ALL".into()]),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    resources: Some(ResourceRequirements {
                        requests: Some(BTreeMap::from([
                            ("cpu".into(), Quantity(self.config.cpu_request.clone())),
                            (
                                "memory".into(),
                                Quantity(self.config.memory_request.clone()),
                            ),
                        ])),
                        limits: Some(BTreeMap::from([
                            ("cpu".into(), Quantity(self.config.cpu_limit.clone())),
                            ("memory".into(), Quantity(self.config.memory_limit.clone())),
                        ])),
                        ..Default::default()
                    }),
                    env: Some(environment),
                    volume_mounts: Some(vec![VolumeMount {
                        name: "saved-environment".into(),
                        mount_path: self.config.saved_mount.clone(),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }],
                volumes: Some(vec![Volume {
                    name: "saved-environment".into(),
                    persistent_volume_claim: Some(PersistentVolumeClaimVolumeSource {
                        claim_name: pvc.name.clone(),
                        read_only: Some(false),
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let created = match api.create(&PostParams::default(), &created).await {
            Ok(created) => created,
            Err(kube::Error::Api(error)) if error.code == 409 => api.get(&pod.name).await?,
            Err(kube::Error::Api(error)) if matches!(error.code, 400 | 401 | 403 | 422 | 429) => {
                return Err(Error::CreationRejected(kube::Error::Api(error)));
            }
            Err(error) => return Err(error.into()),
        };
        self.check_pod(&created, pvc, bootstrap)?;
        self.owned(computer, pod, &created.metadata)
    }
    fn check_pod(
        &self,
        pod: &Pod,
        pvc: &wire::ManagedResource,
        bootstrap: &wire::ManagedResource,
    ) -> Result<(), Error> {
        let spec = pod.spec.as_ref().ok_or(Error::Ownership)?;
        if spec.containers.len() != 1
            || spec.init_containers.as_ref().is_some_and(|v| !v.is_empty())
            || spec
                .ephemeral_containers
                .as_ref()
                .is_some_and(|v| !v.is_empty())
            || spec.host_network == Some(true)
            || spec.host_pid == Some(true)
            || spec.host_ipc == Some(true)
            || spec.restart_policy.as_deref() != Some("Always")
            || !spec.security_context.as_ref().is_some_and(|s| {
                s.run_as_non_root == Some(true)
                    && s.run_as_user == Some(10001)
                    && s.fs_group == Some(10001)
                    && s.seccomp_profile.as_ref().is_some_and(|profile| {
                        profile.type_ == "RuntimeDefault" && profile.localhost_profile.is_none()
                    })
            })
        {
            return Err(Error::Ownership);
        }
        let container = &spec.containers[0];
        if container.image.as_deref() != Some(&self.config.image)
            || spec.service_account_name.as_deref() != Some(&self.config.service_account)
            || spec.automount_service_account_token != Some(false)
            || container.command.as_deref() != Some(&["/opt/chapchoo/chapchoo".into()])
            || container.args.as_deref() != Some(&["computer-connector".into()])
            || !container.security_context.as_ref().is_some_and(|s| {
                s.allow_privilege_escalation == Some(false)
                    && s.read_only_root_filesystem == Some(true)
                    && s.seccomp_profile.as_ref().is_none_or(|profile| {
                        profile.type_ == "RuntimeDefault" && profile.localhost_profile.is_none()
                    })
                    && s.capabilities.as_ref().is_some_and(|c| {
                        c.drop
                            .as_ref()
                            .is_some_and(|d| d.iter().any(|c| c == "ALL"))
                    })
            })
            || !container.volume_mounts.as_ref().is_some_and(|mounts| {
                mounts.len() == 1
                    && mounts[0].name == "saved-environment"
                    && mounts[0].mount_path == self.config.saved_mount
                    && mounts[0].read_only != Some(true)
            })
        {
            return Err(Error::Ownership);
        }
        if !spec.volumes.as_ref().is_some_and(|volumes| {
            volumes.len() == 1
                && volumes.iter().any(|v| {
                    v.name == "saved-environment"
                        && v.persistent_volume_claim.as_ref().is_some_and(|claim| {
                            claim.claim_name == pvc.name && claim.read_only != Some(true)
                        })
                })
        }) {
            return Err(Error::Ownership);
        }
        if !container.env.as_ref().is_some_and(|vars| {
            vars.len() == 3
                && [
                    ("CHAPCHOO_CONNECTOR_INITIAL_KEY", "initial-key"),
                    ("CHAPCHOO_CONNECTOR_SESSION_ID", "session-id"),
                    ("CHAPCHOO_CONTROL_ENDPOINT", "control-endpoint"),
                ]
                .into_iter()
                .all(|(name, key)| {
                    vars.iter().any(|v| {
                        v.name == name
                            && v.value.is_none()
                            && v.value_from
                                .as_ref()
                                .and_then(|v| v.secret_key_ref.as_ref())
                                .is_some_and(|r| {
                                    r.name == bootstrap.name
                                        && r.key == key
                                        && r.optional != Some(true)
                                })
                    })
                })
        }) {
            return Err(Error::Ownership);
        }
        Ok(())
    }
    /// Delete the exact current Pod and observe API-level absence. A fresh 404
    /// or exact deleted event completes retirement without historical events.
    pub async fn retire_pod(
        &self,
        computer: &wire::ComputerContext,
        resource: &wire::ManagedResource,
    ) -> Result<Observation, Error> {
        if self.target(computer, resource)? != ResourceKind::Pod {
            return Err(Error::Ownership);
        }
        if resource.uid.is_none() && resource.absence_confirmed {
            // Current terminal-intent authority can carry committed proof that
            // this candidate was never allocated. A fresh GET must agree; no
            // native deletion or terminal inference is needed for a non-object.
            return self.observe(computer, resource).await;
        }
        let expected = resource.uid.as_ref().ok_or(Error::Ownership)?;
        let api: Api<Pod> = Api::namespaced(self.client.clone(), &resource.namespace);
        let mut observation = Observation {
            resource: resource.clone(),
            presence: Presence::Unknown,
            running: false,
            terminal: false,
        };
        let Some(pod) = api.get_opt(&resource.name).await? else {
            observation.presence = Presence::Absent;
            return Ok(observation);
        };
        self.validate(computer, resource, &pod.metadata)?;
        observation.terminal = terminal_pod(&pod);
        let version = pod
            .metadata
            .resource_version
            .as_ref()
            .ok_or(Error::Ownership)?;
        let params = WatchParams::default()
            .fields(&format!("metadata.name={}", resource.name))
            .timeout(90);
        let mut watch = api.watch(&params, version).await?.boxed();
        let params = DeleteParams {
            preconditions: Some(Preconditions {
                uid: Some(expected.clone()),
                resource_version: None,
            }),
            ..Default::default()
        };
        match api.delete(&resource.name, &params).await {
            Ok(_) => {}
            Err(kube::Error::Api(error)) if error.code == 404 => {
                return self.observe(computer, resource).await;
            }
            Err(error) => return Err(error.into()),
        }
        let result = tokio::time::timeout(std::time::Duration::from_secs(90), async {
            while let Some(event) = watch.try_next().await? {
                let (pod, deleted) = match event {
                    WatchEvent::Added(pod) | WatchEvent::Modified(pod) => (pod, false),
                    WatchEvent::Deleted(pod) => (pod, true),
                    WatchEvent::Bookmark(_) => continue,
                    WatchEvent::Error(_) => return Ok::<_, Error>(false),
                };
                if pod.metadata.uid.as_ref() != Some(expected) {
                    return Ok(false);
                }
                self.validate(computer, resource, &pod.metadata)?;
                observation.terminal |= terminal_pod(&pod);
                if deleted {
                    return Ok(true);
                }
            }
            Ok(false)
        })
        .await;
        if matches!(result, Ok(Ok(true))) {
            observation.presence = Presence::Absent;
        }
        // Native errors after delete submission retain uncertainty, not replay.
        Ok(observation)
    }
    /// Submission is not retirement or absence. Only exact known UID deletion is
    /// allowed, and a native timeout is never retried by this adapter.
    pub async fn delete(
        &self,
        computer: &wire::ComputerContext,
        resource: &wire::ManagedResource,
    ) -> Result<(), Error> {
        let kind = self.target(computer, resource)?;
        let uid = resource.uid.as_ref().ok_or(Error::Ownership)?;
        let observation = self.observe(computer, resource).await?;
        if observation.presence != Presence::Present {
            return Err(Error::Ownership);
        }
        let params = DeleteParams {
            preconditions: Some(Preconditions {
                uid: Some(uid.clone()),
                resource_version: None,
            }),
            ..Default::default()
        };
        match kind {
            ResourceKind::Pod => {
                let api: Api<Pod> = Api::namespaced(self.client.clone(), &resource.namespace);
                api.delete(&resource.name, &params).await?;
            }
            ResourceKind::Pvc => {
                let api: Api<PersistentVolumeClaim> =
                    Api::namespaced(self.client.clone(), &resource.namespace);
                api.delete(&resource.name, &params).await?;
            }
            ResourceKind::BootstrapSecret => {
                let api: Api<Secret> = Api::namespaced(self.client.clone(), &resource.namespace);
                api.delete(&resource.name, &params).await?;
            }
            _ => return Err(Error::Ownership),
        }
        if kind != ResourceKind::Pod {
            // Acknowledgement is only submission. Retain the original task and
            // permit until exact-identity reads establish absence; finalizers
            // never authorize another DELETE submission.
            tokio::time::timeout(std::time::Duration::from_secs(90), async {
                let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    interval.tick().await;
                    match self.observe(computer, resource).await?.presence {
                        Presence::Absent => return Ok(()),
                        Presence::Present => {}
                        Presence::Unknown => return Err(Error::DeletionUncertain),
                    }
                }
            })
            .await
            .map_err(|_| Error::DeletionUncertain)??;
        }
        Ok(())
    }
}

fn terminal_pod(pod: &Pod) -> bool {
    let Some(spec) = pod.spec.as_ref() else {
        return false;
    };
    let Some(status) = pod.status.as_ref() else {
        return false;
    };
    if !matches!(status.phase.as_deref(), Some("Succeeded" | "Failed"))
        || status.reason.as_deref() == Some("NodeLost")
        || spec
            .ephemeral_containers
            .as_ref()
            .is_some_and(|v| !v.is_empty())
    {
        return false;
    }
    let terminated =
        |containers: &[Container],
         statuses: Option<&Vec<k8s_openapi::api::core::v1::ContainerStatus>>| {
            if containers.is_empty() {
                return true;
            }
            let Some(statuses) = statuses else {
                return false;
            };
            statuses.len() == containers.len()
                && containers.iter().all(|container| {
                    statuses.iter().any(|status| {
                        status.name == container.name
                            && status
                                .state
                                .as_ref()
                                .and_then(|s| s.terminated.as_ref())
                                .is_some_and(|state| {
                                    state.container_id.as_ref().is_some_and(|id| !id.is_empty())
                                        && state
                                            .finished_at
                                            .as_ref()
                                            .is_some_and(|t| t.0.as_second() > 0)
                                        && !matches!(
                                            state.reason.as_deref(),
                                            Some("ContainerStatusUnknown" | "NodeLost" | "Unknown")
                                        )
                                })
                    })
                })
        };
    !spec.containers.is_empty()
        && terminated(&spec.containers, status.container_statuses.as_ref())
        && terminated(
            spec.init_containers.as_deref().unwrap_or(&[]),
            status.init_container_statuses.as_ref(),
        )
}
