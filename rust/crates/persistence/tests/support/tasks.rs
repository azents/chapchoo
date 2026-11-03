use super::support::combine;
use tokio::{task::JoinHandle, time::Instant};

type TestError = Box<dyn std::error::Error + Send + Sync>;

/// Race assertions retain typed domain outcomes after every owned task exits.
pub async fn join_pair<A, B, E>(
    mut first: JoinHandle<Result<A, E>>,
    mut second: JoinHandle<Result<B, E>>,
    deadline: Option<Instant>,
) -> Result<(Result<A, E>, Result<B, E>), TestError>
where
    E: Into<TestError>,
{
    // Keep collected completions outside the cancellable timeout future.
    let mut first_exit = None;
    let mut second_exit = None;
    let collect = async {
        tokio::join!(async { first_exit = Some((&mut first).await) }, async {
            second_exit = Some((&mut second).await)
        },);
    };
    let timed_out = if let Some(deadline) = deadline {
        tokio::time::timeout_at(deadline, collect).await.is_err()
    } else {
        collect.await;
        false
    };
    // A database deadline is a failed assertion, not physical cancellation of
    // a started SQLite blocking operation. Retain work through actual exit.
    if first_exit.is_none() {
        first_exit = Some(first.await);
    }
    if second_exit.is_none() {
        second_exit = Some(second.await);
    }
    let first = first_exit.ok_or("Missing first task completion")?;
    let second = second_exit.ok_or("Missing second task completion")?;
    match (first, second) {
        (Ok(first), Ok(second)) if !timed_out => Ok((first, second)),
        (first, second) => {
            let first = match first {
                Ok(result) => result.map(|_| ()).map_err(Into::into),
                Err(error) => Err(error.into()),
            };
            let second = match second {
                Ok(result) => result.map(|_| ()).map_err(Into::into),
                Err(error) => Err(error.into()),
            };
            let failures = combine(first, second).map(|_| ());
            let deadline = if timed_out {
                Err("Concurrent database fixture deadline exceeded".into())
            } else {
                Ok(())
            };
            combine(deadline, failures)?;
            Err("Missing failed task completion".into())
        }
    }
}
