//! Bounded local correlation admission, independently of server capacity policy.
use super::*;
use chapchoo_docker::Bootstrap;
#[tokio::test]
async fn capacity_rejects_before_container_submission_and_reuses_only_a_settled_slot()
-> Result<(), TestError> {
    let fixture = Fixture::new().await?;
    let docker = fixture.connect().await?;
    let saved = docker
        .ensure_volume(&context(), &reserved(false), true)
        .await?;
    let image = docker.prepare_image().await?;
    let bootstrap = Bootstrap {
        session: "session-original",
        initial_key: "original-bootstrap-fixture",
        endpoint: "http://control.invalid:50051",
    };
    let scoped = |index| {
        let mut context = context();
        context.execution_id = format!("execution-{index}");
        let resource = wire::ManagedResource {
            kind: wire::ResourceKind::DockerContainer as i32,
            namespace: String::new(),
            name: "execution-candidate".into(),
            uid: None,
            execution_id: Some(context.execution_id.clone()),
            absence_confirmed: false,
            native: Some(Native::DockerContainer(wire::DockerContainerNative {
                daemon_id: "daemon:fixture".into(),
                container_id: None,
            })),
        };
        (context, resource)
    };
    let mut last = None;
    for index in 0..256 {
        // The fixture materializes one inspect result at a time. The adapter
        // must retain other unresolved/created correlations despite that view.
        fixture.state.lock().map_err(|_| "Fixture lock")?.container = None;
        let (context, resource) = scoped(index);
        let prepared = docker
            .prepare_container(&context, &resource, &saved, &image, &bootstrap, true)
            .await?;
        if index == 0 {
            fixture.state.lock().map_err(|_| "Fixture lock")?.fail_stage = Some("start");
            assert!(docker.start_container(prepared).await.is_err());
            fixture.state.lock().map_err(|_| "Fixture lock")?.fail_stage = None;
        } else if index == 255 {
            last = Some(prepared);
        }
    }
    let last_native = fixture
        .state
        .lock()
        .map_err(|_| "Fixture lock")?
        .container
        .take();
    let (next_context, next_resource) = scoped(256);
    let error = docker
        .prepare_container(
            &next_context,
            &next_resource,
            &saved,
            &image,
            &bootstrap,
            true,
        )
        .await
        .err()
        .ok_or("Capacity must reject before create")?;
    assert!(matches!(error, Error::ContainerCapacity));
    assert!(error.container_creation_retry_safe());
    assert!(
        !error.definitely_rejected(),
        "Local admission is not an Engine rejection"
    );
    {
        let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
        assert_eq!(
            state
                .container_actions
                .iter()
                .filter(|action| *action == "create")
                .count(),
            256
        );
        assert_eq!((state.creates, state.removes), (1, 0));
    }
    // Free only a positively retired exact container. Uncertain/in-flight
    // records must not be evicted to admit another native request.
    fixture.state.lock().map_err(|_| "Fixture lock")?.container = last_native;
    let prepared = last.ok_or("Missing last candidate")?;
    let last_resource = prepared.resource().clone();
    docker.start_container(prepared).await?;
    let (last_context, _) = scoped(255);
    let retiring = docker
        .begin_retirement(
            &last_context,
            &last_resource,
            saved.resource(),
            true,
            true,
            false,
        )
        .await?;
    let updated = docker.disable_restart(retiring).await?;
    let stopped = docker.stop_container(updated).await?;
    assert_eq!(
        docker.remove_container(stopped).await?.presence,
        Presence::Absent
    );
    let recovered = docker
        .ensure_volume(&next_context, saved.resource(), true)
        .await?;
    assert_eq!(recovered.resource(), saved.resource());
    let retried = docker
        .prepare_container(
            &next_context,
            &next_resource,
            &recovered,
            &image,
            &bootstrap,
            true,
        )
        .await?;
    drop(retried);
    // Capacity recovery cannot turn the original unknown Start into another
    // submission, even when the selected native view shows no container.
    fixture.state.lock().map_err(|_| "Fixture lock")?.container = None;
    let (unknown_context, unknown_resource) = scoped(0);
    let unknown = docker
        .prepare_container(
            &unknown_context,
            &unknown_resource,
            &recovered,
            &image,
            &bootstrap,
            true,
        )
        .await
        .err()
        .ok_or("Unknown stage must remain fenced")?;
    assert!(!unknown.container_creation_retry_safe());
    {
        let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
        assert_eq!(
            state
                .container_actions
                .iter()
                .filter(|action| *action == "create")
                .count(),
            257
        );
        assert_eq!((state.creates, state.removes), (1, 0));
        assert!(state.volume.is_some());
    }
    fixture.finish(docker).await
}
