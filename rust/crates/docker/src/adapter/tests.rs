use super::*;

type TestError = Box<dyn std::error::Error + Send + Sync>;

fn config() -> Config {
    Config {
        image: "public.example.invalid/computer:fixture".into(),
        image_policy: ImagePolicy::IfNotPresent,
        saved_mount: "/workspace".into(),
        network: None,
        nano_cpus: 1_000_000_000,
        memory_bytes: 1024 * 1024 * 1024,
    }
}
fn info() -> SystemInfo {
    SystemInfo {
        id: Some("opaque:original-daemon".into()),
        os_type: Some("linux".into()),
        architecture: Some("aarch64".into()),
        memory_limit: Some(true),
        cpu_cfs_period: Some(true),
        cpu_cfs_quota: Some(true),
        security_options: Some(vec!["name=seccomp,profile=builtin".into()]),
        ..Default::default()
    }
}

#[test]
fn daemon_identity_is_opaque_but_missing_or_invalid_facts_are_not_a_new_target()
-> Result<(), TestError> {
    let original = info();
    assert_eq!(identity(&original)?.as_str(), "opaque:original-daemon");
    let mut changed = original.clone();
    changed.id = Some("other-daemon".into());
    assert_ne!(identity(&original)?, identity(&changed)?);
    for id in [
        None,
        Some("".into()),
        Some("invalid daemon".into()),
        Some("x".repeat(1025)),
    ] {
        changed.id = id;
        assert!(identity(&changed).is_err());
    }
    Ok(())
}

#[test]
fn advertised_linux_security_and_limits_are_required_not_silently_dropped() -> Result<(), TestError>
{
    workload_policy(&info())?;
    assert_eq!(architecture(&info())?, "arm64");
    for variant in 0..7 {
        let mut native = info();
        match variant {
            0 => native.os_type = Some("windows".into()),
            1 => native.memory_limit = Some(false),
            2 => native.cpu_cfs_period = None,
            3 => native.cpu_cfs_quota = Some(false),
            4 => native.security_options = Some(vec!["name=seccomp,profile=unconfined".into()]),
            5 => native.security_options = None,
            _ => native.architecture = Some("unsupported".into()),
        }
        assert!(workload_policy(&native).is_err());
    }
    Ok(())
}

#[test]
fn invalid_workload_configuration_does_not_enable_host_or_unbounded_policy() -> Result<(), TestError>
{
    config().validate()?;
    for mount in [
        "relative",
        "/",
        "/workspace/../elsewhere",
        "/workspace/./nested",
        "/workspace/",
    ] {
        let mut configured = config();
        configured.saved_mount = mount.into();
        assert!(configured.validate().is_err());
    }
    for network in ["", "host", "container:another-workload"] {
        let mut configured = config();
        configured.network = Some(network.into());
        assert!(configured.validate().is_err());
    }
    for amount in [0, -1] {
        let mut configured = config();
        configured.nano_cpus = amount;
        assert!(configured.validate().is_err());
        configured = config();
        configured.memory_bytes = amount;
        assert!(configured.validate().is_err());
    }
    assert!(!format!("{:?}", config()).contains("public.example"));
    Ok(())
}
