//! Exact immutable workload profile, independent from native state observations.
use crate::{Docker, Error};
use bollard::models::{
    ContainerCreateBody, ContainerInspectResponse, HostConfig, Mount, MountType,
    MountVolumeOptions, RestartPolicy, RestartPolicyNameEnum,
};
use chapchoo_protocol::control as wire;
use std::{collections::BTreeMap, fmt};

pub struct Bootstrap<'a> {
    pub session: &'a str,
    pub initial_key: &'a str,
    pub endpoint: &'a str,
}
impl fmt::Debug for Bootstrap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DockerBootstrap([REDACTED])")
    }
}
impl Bootstrap<'_> {
    fn environment(&self, saved_root: &str) -> Result<Vec<String>, Error> {
        if [self.session, self.initial_key, self.endpoint]
            .iter()
            .any(|value| {
                value.is_empty() || value.len() > 4096 || value.chars().any(char::is_control)
            })
        {
            return Err(Error::Configuration);
        }
        let url = url::Url::parse(self.endpoint).map_err(|_| Error::Configuration)?;
        if url.scheme() != "http"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
        {
            return Err(Error::Configuration);
        }
        Ok(vec![
            format!("CHAPCHOO_CONNECTOR_SESSION_ID={}", self.session),
            format!("CHAPCHOO_CONNECTOR_INITIAL_KEY={}", self.initial_key),
            format!("CHAPCHOO_CONTROL_ENDPOINT={}", self.endpoint),
            format!("CHAPCHOO_COMPUTER_SAVED_ROOT={saved_root}"),
        ])
    }
}
fn environment(values: &[String]) -> Result<BTreeMap<&str, &str>, Error> {
    let mut selected = BTreeMap::new();
    for value in values {
        let (key, value) = value.split_once('=').ok_or(Error::Ownership)?;
        if key.starts_with("CHAPCHOO_")
            && (value.is_empty()
                || value.len() > 4096
                || value.chars().any(char::is_control)
                || selected.insert(key, value).is_some())
        {
            return Err(Error::Ownership);
        }
    }
    if selected.len() != 4
        || ![
            "CHAPCHOO_CONNECTOR_SESSION_ID",
            "CHAPCHOO_CONNECTOR_INITIAL_KEY",
            "CHAPCHOO_CONTROL_ENDPOINT",
            "CHAPCHOO_COMPUTER_SAVED_ROOT",
        ]
        .iter()
        .all(|key| selected.contains_key(key))
    {
        return Err(Error::Ownership);
    }
    Ok(selected)
}
fn none_or_empty<T>(value: &Option<Vec<T>>) -> bool {
    value.as_ref().is_none_or(Vec::is_empty)
}

impl Docker {
    pub(crate) fn container_profile(
        &self,
        context: &wire::ComputerContext,
        resource: &wire::ManagedResource,
        saved: &wire::ManagedResource,
        bootstrap: &Bootstrap<'_>,
        populate: bool,
    ) -> Result<ContainerCreateBody, Error> {
        self.volume_target(saved)?;
        self.labels(context, saved)?;
        Ok(ContainerCreateBody {
            image: Some(self.config.image.clone()),
            user: Some("10001:10001".into()),
            entrypoint: Some(vec!["/opt/chapchoo/chapchoo".into()]),
            cmd: Some(vec!["computer-connector".into()]),
            env: Some(bootstrap.environment(&self.config.saved_mount)?),
            labels: Some(self.labels(context, resource)?),
            stop_signal: Some("SIGTERM".into()),
            stop_timeout: Some(30),
            host_config: Some(HostConfig {
                memory: Some(self.config.memory_bytes),
                nano_cpus: Some(self.config.nano_cpus),
                network_mode: Some(
                    self.config
                        .network
                        .clone()
                        .unwrap_or_else(|| "bridge".into()),
                ),
                restart_policy: Some(RestartPolicy {
                    name: Some(RestartPolicyNameEnum::ALWAYS),
                    maximum_retry_count: Some(0),
                }),
                auto_remove: Some(false),
                cap_drop: Some(vec!["ALL".into()]),
                readonly_rootfs: Some(true),
                privileged: Some(false),
                security_opt: Some(vec!["no-new-privileges=true".into()]),
                ipc_mode: Some("private".into()),
                publish_all_ports: Some(false),
                mounts: Some(vec![Mount {
                    target: Some(self.config.saved_mount.clone()),
                    source: Some(saved.name.clone()),
                    typ: Some(MountType::VOLUME),
                    read_only: Some(false),
                    volume_options: Some(MountVolumeOptions {
                        no_copy: Some(!populate),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    pub(super) fn validate_container_profile(
        &self,
        target: &super::Target,
        value: &ContainerInspectResponse,
        bootstrap: Option<&Bootstrap<'_>>,
        retiring: bool,
    ) -> Result<(), Error> {
        let context = &target.context;
        let resource = &target.resource;
        let saved = &target.saved;
        let initial = target.initial;
        let config = value.config.as_ref().ok_or(Error::Ownership)?;
        let host = value.host_config.as_ref().ok_or(Error::Ownership)?;
        let policy = host.restart_policy.as_ref().and_then(|policy| policy.name);
        // Image EXPOSE metadata does not publish ports. Enforce publication at
        // HostConfig's PublishAllPorts and PortBindings boundaries below.
        if value.name.as_deref() != Some(format!("/{}", resource.name).as_str())
            || value.image.as_deref().is_none_or(|image| {
                !crate::images::image_digest(image)
                    || target
                        .image
                        .as_deref()
                        .is_some_and(|expected| expected != image)
            })
            || config.labels.as_ref() != Some(&self.labels(context, resource)?)
            || config.image.as_deref() != value.image.as_deref()
            || config.user.as_deref() != Some("10001:10001")
            || config.entrypoint.as_deref() != Some(&["/opt/chapchoo/chapchoo".into()])
            || config.cmd.as_deref() != Some(&["computer-connector".into()])
            || config.stop_signal.as_deref() != Some("SIGTERM")
            || config.stop_timeout != Some(30)
            || config.tty == Some(true)
            || config.open_stdin == Some(true)
            || !none_or_empty(&config.volumes)
            || host.readonly_rootfs != Some(true)
            || host.privileged == Some(true)
            || host.memory != Some(self.config.memory_bytes)
            || host.nano_cpus != Some(self.config.nano_cpus)
            || host.auto_remove == Some(true)
            || host.publish_all_ports == Some(true)
            || host.network_mode.as_deref()
                != Some(self.config.network.as_deref().unwrap_or("bridge"))
            || host.cap_drop.as_deref() != Some(&["ALL".into()])
            || !none_or_empty(&host.cap_add)
            || host.security_opt.as_ref().is_none_or(|values| {
                values.len() != 1
                    || !matches!(
                        values[0].as_str(),
                        "no-new-privileges=true" | "no-new-privileges"
                    )
            })
            || host
                .pid_mode
                .as_ref()
                .is_some_and(|value| !value.is_empty())
            || !matches!(host.ipc_mode.as_deref(), Some("private"))
            || host.init == Some(true)
            || host
                .userns_mode
                .as_ref()
                .is_some_and(|value| !value.is_empty())
            || !none_or_empty(&host.binds)
            || !none_or_empty(&host.volumes_from)
            || !none_or_empty(&host.devices)
            || !none_or_empty(&host.device_requests)
            || !none_or_empty(&host.device_cgroup_rules)
            || host
                .port_bindings
                .as_ref()
                .is_some_and(|values| !values.is_empty())
            || host.tmpfs.as_ref().is_some_and(|values| !values.is_empty())
            || (policy != Some(RestartPolicyNameEnum::ALWAYS)
                && !(retiring && policy == Some(RestartPolicyNameEnum::NO)))
            || host
                .restart_policy
                .as_ref()
                .and_then(|policy| policy.maximum_retry_count)
                .is_some_and(|value| value != 0)
        {
            return Err(Error::Ownership);
        }
        let vars = environment(config.env.as_deref().ok_or(Error::Ownership)?)?;
        if vars.get("CHAPCHOO_COMPUTER_SAVED_ROOT") != Some(&self.config.saved_mount.as_str()) {
            return Err(Error::Ownership);
        }
        if let Some(bootstrap) = bootstrap {
            let expected = bootstrap.environment(&self.config.saved_mount)?;
            if vars != environment(&expected)? {
                return Err(Error::Ownership);
            }
        }
        let mounts = host
            .mounts
            .as_ref()
            .filter(|values| values.len() == 1)
            .ok_or(Error::Ownership)?;
        let mount = &mounts[0];
        if mount.typ != Some(MountType::VOLUME)
            || mount.source.as_deref() != Some(&saved.name)
            || mount.target.as_deref() != Some(&self.config.saved_mount)
            || mount.read_only == Some(true)
            || mount.bind_options.is_some()
            || mount.tmpfs_options.is_some()
            || mount.image_options.is_some()
            || mount.volume_options.as_ref().is_none_or(|options| {
                (!initial && options.no_copy != Some(true))
                    || target
                        .population
                        .is_some_and(|populate| options.no_copy.unwrap_or(false) == populate)
                    || options.driver_config.is_some()
                    || options
                        .subpath
                        .as_ref()
                        .is_some_and(|value| !value.is_empty())
            })
        {
            return Err(Error::Ownership);
        }
        let mounts = value
            .mounts
            .as_ref()
            .filter(|values| values.len() == 1)
            .ok_or(Error::Ownership)?;
        let actual = &mounts[0];
        if actual.typ.as_ref().map(AsRef::as_ref) != Some("volume")
            || actual.name.as_deref() != Some(&saved.name)
            || actual.destination.as_deref() != Some(&self.config.saved_mount)
            || actual.rw != Some(true)
            || actual.driver.as_deref() != Some("local")
        {
            return Err(Error::Ownership);
        }
        Ok(())
    }
}
