use super::*;
#[tokio::test]
async fn public_image_policy_consumes_errors_and_never_silently_emulates_or_changes_network()
-> Result<(), TestError> {
    for (policy, present, error, pulls, success) in [
        (ImagePolicy::Never, true, false, 0, true),
        (ImagePolicy::Never, false, false, 0, false),
        (ImagePolicy::IfNotPresent, true, false, 0, true),
        (ImagePolicy::IfNotPresent, false, false, 1, true),
        (ImagePolicy::Always, true, false, 1, true),
        (ImagePolicy::Always, true, true, 1, false),
    ] {
        let fixture = Fixture::new().await?;
        {
            let mut state = fixture.state.lock().map_err(|_| "Fixture lock")?;
            state.image_present = present;
            state.image_error = error;
        }
        let docker = fixture.connect_policy(policy).await?;
        assert_eq!(docker.prepare_image().await.is_ok(), success);
        {
            let state = fixture.state.lock().map_err(|_| "Fixture lock")?;
            assert_eq!(state.image_pulls, pulls);
            assert!(state.container_actions.is_empty());
        }
        fixture.finish(docker).await?;
    }
    for wrong_architecture in [false, true] {
        let fixture = Fixture::new().await?;
        {
            let mut state = fixture.state.lock().map_err(|_| "Fixture lock")?;
            if wrong_architecture {
                state.image_architecture = "amd64".into();
            } else {
                state.network_internal = true;
            }
        }
        let docker = fixture.connect().await?;
        assert!(docker.prepare_image().await.is_err());
        assert_eq!(
            fixture
                .state
                .lock()
                .map_err(|_| "Fixture lock")?
                .image_pulls,
            0
        );
        fixture.finish(docker).await?;
    }
    Ok(())
}
