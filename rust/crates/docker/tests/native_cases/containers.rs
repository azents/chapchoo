use super::*;
use chapchoo_docker::Bootstrap;
fn container() -> wire::ManagedResource {
    wire::ManagedResource {
        kind: wire::ResourceKind::DockerContainer as i32,
        namespace: String::new(),
        name: "execution-candidate".into(),
        uid: None,
        execution_id: Some("execution".into()),
        allocation: Some(wire::NativeAllocationWitness {
            computer_id: "computer".into(),
            provider_id: "provider".into(),
            execution_id: Some("execution".into()),
        }),
        absence_confirmed: false,
        native: Some(Native::DockerContainer(wire::DockerContainerNative {
            daemon_id: "daemon:fixture".into(),
            container_id: None,
        })),
    }
}
fn bootstrap() -> Bootstrap<'static> {
    Bootstrap {
        session: "session-original",
        initial_key: "original-bootstrap-fixture",
        endpoint: "http://control.invalid:50051",
    }
}
async fn prepare(
    fixture: &Fixture,
    docker: &Docker,
) -> Result<chapchoo_docker::PreparedContainer, TestError> {
    fixture.state.lock().map_err(|_| "Fixture lock")?.volume = Some(volume());
    let saved = docker
        .ensure_volume(&context(), &reserved(true), true)
        .await?;
    Ok(docker
        .prepare_container(
            &context(),
            &container(),
            &saved,
            &docker.prepare_image().await?,
            &bootstrap(),
            true,
        )
        .await?)
}

#[tokio::test]
async fn only_a_fresh_volume_creation_may_populate_even_for_initial_execution_retry()
-> Result<(), TestError> {
    for existing_original in [false, true] {
        Fixture::run(async |fixture| {
            let docker = fixture.connect().await?;
            if existing_original {
                fixture.state.lock().map_err(|_| "Fixture lock")?.volume = Some(volume());
            }
            let saved = docker
                .ensure_volume(&context(), &reserved(existing_original), true)
                .await?;
            let prepared = docker
                .prepare_container(
                    &context(),
                    &container(),
                    &saved,
                    &docker.prepare_image().await?,
                    &bootstrap(),
                    true,
                )
                .await?;
            {
                let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
                let no_copy = state
                    .container
                    .as_ref()
                    .and_then(|value| value.host_config.as_ref())
                    .and_then(|host| host.mounts.as_ref())
                    .and_then(|mounts| mounts.first())
                    .and_then(|mount| mount.volume_options.as_ref())
                    .and_then(|options| options.no_copy);
                assert_eq!(no_copy, Some(existing_original));
                assert_eq!(state.creates, usize::from(!existing_original));
            }
            drop(prepared);
            Ok(())
        })
        .await?;
    }
    Ok(())
}

#[tokio::test]
async fn normal_native_lifecycle_submits_each_stage_once_and_keeps_original_saved_volume()
-> Result<(), TestError> {
    Fixture::run(async |fixture| {
        let docker = fixture.connect().await?;
        let prepared = prepare(fixture, &docker).await?;
        let resource = prepared.resource().clone();
        assert!(docker.start_container(prepared).await?.running);
        // Request replacement must not discard the same execution's native facts.
        let context = wire::ComputerContext {
            request_id: "stop".into(),
            ..context()
        };
        let retirement = docker
            .begin_retirement(&context, &resource, &reserved(true), true, true, false)
            .await?;
        let updated = docker.disable_restart(retirement).await?;
        let stopped = docker.stop_container(updated).await?;
        assert_eq!(
            docker.remove_container(stopped).await?.presence,
            Presence::Absent
        );
        assert_eq!(
            docker
                .observe_container(&context, &resource, &reserved(true), true, true)
                .await?
                .presence,
            Presence::Absent
        );
        {
            let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
            assert_eq!(
                state.container_actions,
                ["create", "start", "update", "stop", "remove"]
            );
            assert!(state.volume.is_some());
            assert_eq!(state.removes, 0);
        }
        Ok(())
    })
    .await
}

#[tokio::test]
async fn intentional_retirement_stops_restarting_work_without_claiming_running_or_terminal()
-> Result<(), TestError> {
    for restart_at in 0..3 {
        Fixture::run(async |fixture| {
            let docker = fixture.connect().await?;
            let prepared = prepare(fixture, &docker).await?;
            let resource = prepared.resource().clone();
            docker.start_container(prepared).await?;
            let enter_restart = || -> Result<(), TestError> {
                let mut shared = fixture.state.lock().map_err(|_| "Fixture lock")?;
                let state = shared
                    .container
                    .as_mut()
                    .and_then(|value| value.state.as_mut())
                    .ok_or("Missing native state")?;
                state.status = Some(ContainerStateStatusEnum::RESTARTING);
                state.running = Some(true);
                state.restarting = Some(true);
                state.pid = Some(0);
                state.exit_code = Some(1);
                state.finished_at = Some("2026-11-03T06:47:00Z".into());
                Ok(())
            };
            if restart_at == 0 {
                enter_restart()?;
                let observed = docker
                    .observe_container(&context(), &resource, &reserved(true), true, true)
                    .await?;
                assert!(!observed.running && !observed.terminal);
                assert!(
                    docker
                        .begin_retirement(&context(), &resource, &reserved(true), true, true, true)
                        .await
                        .is_err()
                );
            }
            let retirement = docker
                .begin_retirement(&context(), &resource, &reserved(true), true, true, false)
                .await?;
            if restart_at == 1 {
                enter_restart()?;
            }
            let updated = docker.disable_restart(retirement).await?;
            if restart_at == 2 {
                enter_restart()?;
            }
            let stopped = docker.stop_container(updated).await?;
            assert_eq!(
                docker.remove_container(stopped).await?.presence,
                Presence::Absent
            );
            let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
            assert_eq!(
                state.container_actions,
                ["create", "start", "update", "stop", "remove"]
            );
            assert!(state.volume.is_some());
            assert_eq!(state.removes, 0);
            Ok(())
        })
        .await?;
    }
    Ok(())
}

#[tokio::test]
async fn contradictory_restarting_state_cannot_admit_intentional_retirement()
-> Result<(), TestError> {
    for variant in 0..7 {
        Fixture::run(async |fixture| {
            let docker = fixture.connect().await?;
            let prepared = prepare(fixture, &docker).await?;
            let resource = prepared.resource().clone();
            docker.start_container(prepared).await?;
            {
                let mut shared = fixture.state.lock().map_err(|_| "Fixture lock")?;
                let state = shared
                    .container
                    .as_mut()
                    .and_then(|value| value.state.as_mut())
                    .ok_or("Missing native state")?;
                state.status = Some(ContainerStateStatusEnum::RESTARTING);
                state.restarting = Some(true);
                state.pid = Some(0);
                state.finished_at = Some("2026-11-03T06:47:00Z".into());
                match variant {
                    0 => state.running = Some(false),
                    1 => state.paused = Some(true),
                    2 => state.dead = Some(true),
                    3 => state.pid = Some(123),
                    4 => state.status = Some(ContainerStateStatusEnum::RUNNING),
                    5 => state.finished_at = Some("invalid-time".into()),
                    _ => state.started_at = Some("2026-11-03T06:48:00Z".into()),
                }
            }
            assert!(
                docker
                    .begin_retirement(&context(), &resource, &reserved(true), true, true, false)
                    .await
                    .is_err()
            );
            assert_eq!(
                fixture
                    .state
                    .lock()
                    .map_err(|_| "Fixture lock")?
                    .container_actions,
                ["create", "start"]
            );
            Ok(())
        })
        .await?;
    }
    Ok(())
}

#[tokio::test]
async fn image_expose_metadata_is_not_host_port_publication() -> Result<(), TestError> {
    Fixture::run(async |fixture| {
        let docker = fixture.connect().await?;
        let prepared = prepare(fixture, &docker).await?;
        {
            let mut state = fixture.state.lock().map_err(|_| "Fixture lock")?;
            let value = state.container.as_mut().ok_or("Container")?;
            // The standard backend image declares these ports as image metadata.
            value.config.as_mut().ok_or("Config")?.exposed_ports =
                Some(vec!["50051/tcp".into(), "50053/tcp".into()]);
        }
        assert!(docker.start_container(prepared).await?.running);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn inherited_image_metadata_does_not_replace_native_ownership() -> Result<(), TestError> {
    Fixture::run(async |fixture| {
        let docker = fixture.connect().await?;
        let prepared = prepare(fixture, &docker).await?;
        {
            let mut state = fixture.state.lock().map_err(|_| "Fixture lock")?;
            let labels = state
                .container
                .as_mut()
                .and_then(|container| container.config.as_mut())
                .and_then(|config| config.labels.as_mut())
                .ok_or("Missing labels")?;
            labels.insert("org.opencontainers.image.title".into(), "Chapchoo".into());
            labels.insert(
                "org.opencontainers.image.revision".into(),
                "isolated".into(),
            );
        }
        assert!(docker.start_container(prepared).await?.running);
        Ok(())
    })
    .await
}

#[tokio::test]
async fn missing_changed_or_extra_reserved_labels_never_authorize_native_start()
-> Result<(), TestError> {
    for variant in 0..3 {
        Fixture::run(async |fixture| {
            let docker = fixture.connect().await?;
            let prepared = prepare(fixture, &docker).await?;
            {
                let mut state = fixture.state.lock().map_err(|_| "Fixture lock")?;
                let labels = state
                    .container
                    .as_mut()
                    .and_then(|container| container.config.as_mut())
                    .and_then(|config| config.labels.as_mut())
                    .ok_or("Missing labels")?;
                labels.insert("org.opencontainers.image.title".into(), "Chapchoo".into());
                match variant {
                    0 => {
                        labels.remove("chapchoo.io/execution");
                    }
                    1 => {
                        labels.insert("chapchoo.io/provider".into(), "foreign".into());
                    }
                    _ => {
                        labels.insert("chapchoo.io/unknown".into(), "foreign".into());
                    }
                }
            }
            assert!(matches!(
                docker.start_container(prepared).await,
                Err(Error::Ownership)
            ));
            assert_eq!(
                fixture
                    .state
                    .lock()
                    .map_err(|_| "Fixture lock")?
                    .container_actions,
                ["create"]
            );
            Ok(())
        })
        .await?;
    }
    Ok(())
}

#[tokio::test]
async fn operational_and_undefined_image_labels_never_authorize_native_start()
-> Result<(), TestError> {
    for label in [
        "traefik.enable",
        "com.centurylinklabs.watchtower.enable",
        "io.portainer.accesscontrol.teams",
        "fixture.example/build",
        "org.opencontainers.image.routing",
    ] {
        Fixture::run(async |fixture| {
            let docker = fixture.connect().await?;
            let prepared = prepare(fixture, &docker).await?;
            fixture
                .state
                .lock()
                .map_err(|_| "Fixture lock")?
                .container
                .as_mut()
                .and_then(|container| container.config.as_mut())
                .and_then(|config| config.labels.as_mut())
                .ok_or("Missing labels")?
                .insert(label.into(), "enabled".into());
            assert!(matches!(
                docker.start_container(prepared).await,
                Err(Error::Ownership)
            ));
            assert_eq!(
                fixture
                    .state
                    .lock()
                    .map_err(|_| "Fixture lock")?
                    .container_actions,
                ["create"]
            );
            Ok(())
        })
        .await?;
    }
    Ok(())
}

#[tokio::test]
async fn ambiguous_native_start_cannot_be_cleared_by_running_observation_or_old_server_fact()
-> Result<(), TestError> {
    Fixture::run(async |fixture| {
        let docker = fixture.connect().await?;
        let prepared = prepare(fixture, &docker).await?;
        let resource = prepared.resource().clone();
        fixture.state.lock().map_err(|_| "Fixture lock")?.fail_stage = Some("start");
        assert!(docker.start_container(prepared).await.is_err());
        assert!(
            !docker
                .observe_container(&context(), &resource, &reserved(true), true, true)
                .await?
                .running
        );
        assert!(
            docker
                .begin_retirement(&context(), &resource, &reserved(true), true, true, false)
                .await
                .is_err()
        );
        assert!(
            docker
                .prepare_container(
                    &context(),
                    &container(),
                    &docker
                        .ensure_volume(&context(), &reserved(true), true)
                        .await?,
                    &docker.prepare_image().await?,
                    &bootstrap(),
                    true
                )
                .await
                .is_err()
        );
        assert_eq!(
            fixture
                .state
                .lock()
                .map_err(|_| "Fixture lock")?
                .container_actions,
            ["create", "start"]
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn ambiguous_update_stop_or_remove_never_replays_or_advances_other_effects()
-> Result<(), TestError> {
    for stage in ["update", "stop", "remove"] {
        Fixture::run(async |fixture| {
            let docker = fixture.connect().await?;
            let prepared = prepare(fixture, &docker).await?;
            let resource = prepared.resource().clone();
            docker.start_container(prepared).await?;
            fixture.state.lock().map_err(|_| "Fixture lock")?.fail_stage = Some(stage);
            let retirement = docker
                .begin_retirement(&context(), &resource, &reserved(true), true, true, false)
                .await?;
            let outcome = match docker.disable_restart(retirement).await {
                Err(error) => Err(error),
                Ok(updated) => match docker.stop_container(updated).await {
                    Err(error) => Err(error),
                    Ok(stopped) => docker.remove_container(stopped).await.map(|_| ()),
                },
            };
            assert!(outcome.is_err());
            assert!(
                docker
                    .begin_retirement(&context(), &resource, &reserved(true), true, true, false)
                    .await
                    .is_err()
            );
            let observed = docker
                .observe_container(&context(), &resource, &reserved(true), true, true)
                .await?;
            assert!(!observed.running);
            if stage == "remove" {
                assert_eq!(observed.presence, Presence::Unknown);
            }
            {
                let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
                let expected = match stage {
                    "update" => vec!["create", "start", "update"],
                    "stop" => vec!["create", "start", "update", "stop"],
                    _ => vec!["create", "start", "update", "stop", "remove"],
                };
                assert_eq!(state.container_actions, expected);
            }
            Ok(())
        })
        .await?;
    }
    Ok(())
}

#[tokio::test]
async fn paused_restarting_dead_or_always_exited_facts_never_prove_power_or_terminal_retirement()
-> Result<(), TestError> {
    Fixture::run(async |fixture| {
        let docker = fixture.connect().await?;
        let prepared = prepare(fixture, &docker).await?;
        let resource = prepared.resource().clone();
        docker.start_container(prepared).await?;
        let original = fixture
            .state
            .lock()
            .map_err(|_| "Fixture lock")?
            .container
            .as_ref()
            .and_then(|value| value.state.clone())
            .ok_or("Missing native state")?;
        for variant in 0..8 {
            {
                let mut shared = fixture.state.lock().map_err(|_| "Fixture lock")?;
                let mut state = original.clone();
                match variant {
                    0 => state.paused = Some(true),
                    1 => state.restarting = Some(true),
                    2 => state.dead = Some(true),
                    3 => state.pid = Some(0),
                    4 => state.started_at = Some("invalid-time".into()),
                    5 => state.finished_at = Some("invalid-finish".into()),
                    6 => state.finished_at = None,
                    _ => {
                        state.status = Some(ContainerStateStatusEnum::EXITED);
                        state.running = Some(false);
                        state.pid = Some(0);
                        state.finished_at = Some("2026-11-03T06:47:00Z".into());
                    }
                }
                shared.container.as_mut().ok_or("Missing container")?.state = Some(state);
            }
            let observed = docker
                .observe_container(&context(), &resource, &reserved(true), true, true)
                .await?;
            assert!(!observed.running && !observed.terminal);
            assert!(
                docker
                    .begin_retirement(&context(), &resource, &reserved(true), true, true, true)
                    .await
                    .is_err()
            );
        }
        assert_eq!(
            fixture
                .state
                .lock()
                .map_err(|_| "Fixture lock")?
                .container_actions,
            ["create", "start"]
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn mismatched_bootstrap_mount_security_and_full_id_are_never_adopted() -> Result<(), TestError>
{
    Fixture::run(async |fixture| {
        let docker = fixture.connect().await?;
        let prepared = prepare(fixture, &docker).await?;
        let resource = prepared.resource().clone();
        let original = fixture
            .state
            .lock()
            .map_err(|_| "Fixture lock")?
            .container
            .clone()
            .ok_or("Missing container")?;
        drop(prepared);
        for variant in 0..14 {
            let mut value = original.clone();
            match variant {
                0 => value
                    .config
                    .as_mut()
                    .ok_or("Config")?
                    .env
                    .as_mut()
                    .ok_or("Env")?
                    .push("CHAPCHOO_PROVIDER_OWNER_TOKEN=foreign".into()),
                1 => {
                    value.config.as_mut().ok_or("Config")?.env = Some(vec![
                        "CHAPCHOO_CONNECTOR_SESSION_ID=wrong".into(),
                        "CHAPCHOO_CONNECTOR_INITIAL_KEY=wrong".into(),
                        "CHAPCHOO_CONTROL_ENDPOINT=http://control.invalid:50051".into(),
                    ])
                }
                2 => value.host_config.as_mut().ok_or("Host")?.privileged = Some(true),
                3 => value.host_config.as_mut().ok_or("Host")?.network_mode = Some("host".into()),
                4 => {
                    value.mounts.as_mut().ok_or("Mount")?[0].name =
                        Some("replacement-empty-volume".into())
                }
                5 => value.id = Some("b".repeat(64)),
                6 => {
                    value
                        .host_config
                        .as_mut()
                        .ok_or("Host")?
                        .mounts
                        .as_mut()
                        .ok_or("Mount")?[0]
                        .volume_options
                        .as_mut()
                        .ok_or("Options")?
                        .no_copy = Some(false)
                }
                7 => value.image = Some(format!("sha256:{}", "d".repeat(64))),
                8 => value.image = Some(format!("sha256:{}", "z".repeat(64))),
                9 => value.host_config.as_mut().ok_or("Host")?.publish_all_ports = Some(true),
                10 => {
                    value.host_config.as_mut().ok_or("Host")?.port_bindings =
                        Some([("50051/tcp".into(), None)].into_iter().collect())
                }
                11 => value.host_config.as_mut().ok_or("Host")?.init = Some(true),
                12 => value.host_config.as_mut().ok_or("Host")?.pid_mode = Some("host".into()),
                _ => {
                    let env = value
                        .config
                        .as_mut()
                        .ok_or("Config")?
                        .env
                        .as_mut()
                        .ok_or("Env")?;
                    let saved = env
                        .iter_mut()
                        .find(|v| v.starts_with("CHAPCHOO_COMPUTER_SAVED_ROOT="))
                        .ok_or("Saved root")?;
                    *saved = "CHAPCHOO_COMPUTER_SAVED_ROOT=/another-root".into();
                }
            }
            fixture.state.lock().map_err(|_| "Fixture lock")?.container = Some(value);
            assert!(
                docker
                    .prepare_container(
                        &context(),
                        &resource,
                        &docker
                            .ensure_volume(&context(), &reserved(true), true)
                            .await?,
                        &docker.prepare_image().await?,
                        &bootstrap(),
                        true
                    )
                    .await
                    .is_err()
            );
        }
        assert_eq!(
            fixture
                .state
                .lock()
                .map_err(|_| "Fixture lock")?
                .container_actions,
            ["create"]
        );
        Ok(())
    })
    .await
}
