//! Coalesced same-applied-record renewal, separate from native command execution.
use super::*;
use chapchoo_harness::{Error, NativeHandle};
use std::sync::Arc;
use tokio::task::JoinSet;
pub(super) struct AuthenticationSync {
    native: NativeHandle,
    tasks: JoinSet<Result<(), Error>>,
    stop: CancellationToken,
    next: tokio::time::Instant,
}
impl AuthenticationSync {
    pub(super) fn new(native: NativeHandle) -> Self {
        Self {
            native,
            tasks: JoinSet::new(),
            stop: CancellationToken::new(),
            next: tokio::time::Instant::now(),
        }
    }
    pub(super) fn schedule(
        &mut self,
        channel: Channel,
        origin: chapchoo_protocol::conversations::CaptureSource,
        key: &BearerSecret,
    ) -> Result<(), AppError> {
        while let Some(result) = self.tasks.try_join_next() {
            if result?.is_err() {
                tracing::warn!("Native credential file synchronization could not be completed");
            }
        }
        if !self.tasks.is_empty()
            || self.stop.is_cancelled()
            || tokio::time::Instant::now() < self.next
        {
            return Ok(());
        }
        self.next = tokio::time::Instant::now() + Duration::from_secs(5);
        let native = self.native.clone();
        let key = BearerSecret::parse(key.expose().to_owned())?;
        let stop = self.stop.clone();
        self.tasks.spawn(async move {
            // This read does not start native work or create an applied identity.
            let expected=tokio::select! {
                _=stop.cancelled()=>return Ok(()),
                value=native.authentication_binding()=>match value? {Some(binding)=>binding,None=>return Ok(())},
            };
            let provider=Arc::new(authentication::Authentication::for_applied(channel,origin,&key,&expected)
                .map_err(|_|Error::Authentication)?);
            let update=tokio::select! {
                _=stop.cancelled()=>return Ok(()),
                value=provider.current(&expected)=>match value? {Some(update)=>update,None=>return Ok(())},
            };
            // Same-record only. The actor checks current identity/source and
            // final server authority, then publishes only the file.
            tokio::select! {
                _=stop.cancelled()=>Ok(()),
                value=native.publish_authentication(expected,update,provider)=>value,
            }
        });
        Ok(())
    }
    pub(super) async fn finish(mut self) -> Result<(), AppError> {
        self.stop.cancel();
        let mut result = Ok(());
        while let Some(completed) = self.tasks.join_next().await {
            match completed {
                Ok(Err(_)) => tracing::warn!(
                    "Native credential synchronization ended before file publication"
                ),
                Ok(Ok(())) => {}
                Err(error) => result = crate::completion::complete(result, Err(error.into())),
            }
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn idle_connectivity_coalesces_without_contacting_credentials_or_starting_native()
    -> Result<(), AppError> {
        let saved = tempfile::tempdir()?;
        let native = chapchoo_harness::NativeOwner::new(chapchoo_harness::NativeStorage::open(
            saved.path(),
        )?)?;
        let mut owner = AuthenticationSync::new(native.handle());
        let channel = tonic::transport::Endpoint::from_static("http://127.0.0.1:1").connect_lazy();
        let source = chapchoo_protocol::conversations::CaptureSource {
            session_id: "unused-session".into(),
            control_id: "unused-control".into(),
            connection_id: "unused-route".into(),
        };
        let key = BearerSecret::generate()?;
        owner.schedule(channel.clone(), source.clone(), &key)?;
        owner.schedule(channel, source, &key)?;
        assert_eq!(owner.tasks.len(), 1);
        owner
            .tasks
            .join_next()
            .await
            .ok_or("Missing synchronization work")???;
        owner.finish().await?;
        assert_eq!(
            native.finish().await?,
            chapchoo_harness::Retirement::NoNativeStarted
        );
        assert!(
            !saved
                .path()
                .join(".chapchoo-native/state/auth.json")
                .exists()
        );
        Ok(())
    }
}
