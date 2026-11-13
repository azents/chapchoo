//! Exact live-application reads and serialized file-only publication.
use super::*;
impl NativeHandle {
    pub async fn authentication_binding(
        &self,
    ) -> Result<Option<NativeAuthenticationBinding>, Error> {
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::AuthenticationBinding(result))
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
    /// File publication only; not authentication adoption or native login.
    pub async fn publish_authentication(
        &self,
        expected: NativeAuthenticationBinding,
        update: crate::NativeAuthentication,
        checker: Arc<dyn NativeAuthenticationProvider>,
    ) -> Result<(), Error> {
        let bytes =
            u32::try_from(update.credentials.payload_len()).map_err(|_| Error::NotAdmitted)?;
        let permit = self
            .payload_budget
            .clone()
            .try_acquire_many_owned(bytes)
            .map_err(|_| Error::NotAdmitted)?;
        let (result, rx) = oneshot::channel();
        self.sender
            .try_send(Command::PublishAuthentication {
                expected,
                update,
                checker,
                _permit: permit,
                result,
            })
            .map_err(|_| Error::NotAdmitted)?;
        rx.await.map_err(|_| Error::Process)?
    }
}
impl State {
    pub(super) fn current_authentication(
        &self,
    ) -> Result<Option<NativeAuthenticationBinding>, Error> {
        if self.blocked {
            return Err(Error::NotAdmitted);
        }
        Ok(self
            .live
            .as_ref()
            .filter(|live| !live.protocol.is_finished())
            .map(|live| live._authentication.clone()))
    }
    pub(super) async fn publish_authentication(
        &mut self,
        expected: NativeAuthenticationBinding,
        update: crate::NativeAuthentication,
        checker: Arc<dyn NativeAuthenticationProvider>,
    ) -> Result<(), Error> {
        if self.current_authentication()?.as_ref() != Some(&expected)
            || !same_application(&expected, &update.binding)
        {
            return Err(Error::Authentication);
        }
        if update.binding.source == expected.source {
            return Ok(());
        }
        // Acquisition/refresh is outside this actor. Only the bounded final
        // authority check and serialized file publication occur here.
        checker.check(&update.binding).await?;
        if self.current_authentication()?.as_ref() != Some(&expected) {
            return Err(Error::Authentication);
        }
        let storage = self.storage.clone();
        let binding = update.binding;
        self.credential_write = Some(tokio::task::spawn_blocking(move || {
            storage.publish_credentials(&update.credentials)?;
            Ok(update.credentials)
        }));
        let written = self.credential_write.as_mut().ok_or(Error::Process)?.await;
        self.credential_write = None;
        written.map_err(|_| Error::Storage)??;
        self.live
            .as_mut()
            .ok_or(Error::NotAdmitted)?
            ._authentication = binding;
        Ok(())
    }
}
pub(super) fn same_application(
    one: &NativeAuthenticationBinding,
    two: &NativeAuthenticationBinding,
) -> bool {
    one.agent_id == two.agent_id
        && one.conversation_id == two.conversation_id
        && one.operation_id == two.operation_id
        && one.execution_id == two.execution_id
        && one.authentication_id == two.authentication_id
}
