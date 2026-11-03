use super::*;
use chapchoo_docker::Bootstrap;
fn container() -> wire::ManagedResource {
    wire::ManagedResource {
        kind: wire::ResourceKind::DockerContainer as i32,
        namespace: String::new(),
        name: "execution-candidate".into(),
        uid: None,
        execution_id: Some("execution".into()),
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
        let fixture = Fixture::new().await?;
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
        fixture.finish(docker).await?;
    }
    Ok(())
}

#[tokio::test]
async fn normal_native_lifecycle_submits_each_stage_once_and_keeps_original_saved_volume()
-> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    let prepared = prepare(&fixture, &docker).await?;
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
    fixture.finish(docker).await
}

#[tokio::test]
async fn ambiguous_native_start_cannot_be_cleared_by_running_observation_or_old_server_fact()
-> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    let prepared = prepare(&fixture, &docker).await?;
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
    fixture.finish(docker).await
}

#[tokio::test]
async fn ambiguous_update_stop_or_remove_never_replays_or_advances_other_effects()
-> Result<(), TestError> {
    for stage in ["update", "stop", "remove"] {
        let fixture = Fixture::new().await?;
        let docker = fixture.connect().await?;
        let prepared = prepare(&fixture, &docker).await?;
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
        fixture.finish(docker).await?;
    }
    Ok(())
}

#[tokio::test]
async fn paused_restarting_dead_or_always_exited_facts_never_prove_power_or_terminal_retirement()
-> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    let prepared = prepare(&fixture, &docker).await?;
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
    fixture.finish(docker).await
}

#[tokio::test]
async fn mismatched_bootstrap_mount_security_and_full_id_are_never_adopted() -> Result<(), TestError>
{
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    let prepared = prepare(&fixture, &docker).await?;
    let resource = prepared.resource().clone();
    let original = fixture
        .state
        .lock()
        .map_err(|_| "Fixture lock")?
        .container
        .clone()
        .ok_or("Missing container")?;
    drop(prepared);
    for variant in 0..9 {
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
            _ => value.image = Some(format!("sha256:{}", "z".repeat(64))),
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
    fixture.finish(docker).await
}
