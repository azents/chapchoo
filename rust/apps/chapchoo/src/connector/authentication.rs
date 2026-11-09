//! Actual native-start credential acquisition over the existing component route.
use super::*;
use chapchoo_crypto::ExternalSecret;
use chapchoo_harness::{
    Error, NativeAuthentication, NativeAuthenticationBinding, NativeAuthenticationFuture,
    NativeAuthenticationProvider, NativeCredentials,
};
use chapchoo_protocol::conversations::{
    self as c, connector_capture_client::ConnectorCaptureClient,
};

pub(super) struct Authentication {
    channel: Channel,
    target: c::ExecutionAuthenticationReference,
    key: BearerSecret,
}
impl Authentication {
    pub(super) fn for_applied(
        channel: Channel,
        origin: c::CaptureSource,
        key: &BearerSecret,
        binding: &NativeAuthenticationBinding,
    ) -> Result<Self, AppError> {
        Ok(Self {
            channel,
            key: BearerSecret::parse(key.expose().to_owned())?,
            target: c::ExecutionAuthenticationReference {
                source: Some(origin),
                agent_id: binding.agent_id.clone(),
                conversation_id: binding.conversation_id.clone(),
                operation_id: binding.operation_id.clone(),
            },
        })
    }
    pub(super) async fn current(
        &self,
        binding: &NativeAuthenticationBinding,
    ) -> Result<Option<NativeAuthentication>, Error> {
        self.retrieve(
            c::AppliedAuthentication {
                operation_id: binding.operation_id.clone(),
                execution_id: binding.execution_id.clone(),
                authentication_id: binding.authentication_id.clone(),
            },
            Some(binding.source),
        )
        .await
    }
    pub(super) fn new(
        channel: Channel,
        origin: c::CaptureSource,
        key: &BearerSecret,
        command: &c::SetupCommand,
    ) -> Result<Self, AppError> {
        Ok(Self {
            channel,
            target: c::ExecutionAuthenticationReference {
                source: Some(origin),
                agent_id: command.agent_id.clone(),
                conversation_id: command.conversation_id.clone(),
                operation_id: command.operation_id.clone(),
            },
            key: BearerSecret::parse(key.expose().to_owned())?,
        })
    }
    fn client(&self) -> ConnectorCaptureClient<Channel> {
        ConnectorCaptureClient::new(self.channel.clone())
            .max_decoding_message_size(64 * 1024)
            .max_encoding_message_size(64 * 1024)
    }
    fn request<T>(&self, value: T) -> Result<Request<T>, Error> {
        request(value, &self.key).map_err(|_| Error::Authentication)
    }
    async fn acquire_once(&self) -> Result<NativeAuthentication, Error> {
        let mut client = self.client();
        let previous = client
            .get_authentication_binding(self.request(self.target.clone())?)
            .await
            .map_err(|_| Error::Authentication)?
            .into_inner()
            .applied;
        let applied = client
            .begin_authentication(self.request(c::BeginAuthenticationRequest {
                target: Some(self.target.clone()),
                expected_previous_operation_id: previous.map(|value| value.operation_id),
            })?)
            .await
            .map_err(|_| Error::Authentication)?
            .into_inner();
        self.retrieve(applied, None)
            .await?
            .ok_or(Error::Authentication)
    }
    async fn retrieve(
        &self,
        applied: c::AppliedAuthentication,
        unchanged: Option<[u8; 24]>,
    ) -> Result<Option<NativeAuthentication>, Error> {
        let mut client = self.client();
        let progress = client
            .acquire_authentication(self.request(self.target.clone())?)
            .await
            .map_err(|_| Error::Authentication)?
            .into_inner();
        let Some(c::execution_authentication_progress::State::Current(descriptor)) = progress.state
        else {
            return Err(Error::Authentication);
        };
        if descriptor.applied.as_ref() != Some(&applied)
            || descriptor.total_bytes == 0
            || descriptor.total_bytes > 65536
        {
            return Err(Error::Authentication);
        }
        let source: [u8; 24] = descriptor
            .credential_source
            .as_slice()
            .try_into()
            .map_err(|_| Error::Authentication)?;
        let mut bytes =
            zeroize::Zeroizing::new(Vec::with_capacity(descriptor.total_bytes as usize));
        if unchanged == Some(source) {
            return Ok(None);
        }
        while bytes.len() < descriptor.total_bytes as usize {
            let offset = u32::try_from(bytes.len()).map_err(|_| Error::Authentication)?;
            let piece = client
                .read_authentication(self.request(c::AuthenticationReadRequest {
                    target: Some(self.target.clone()),
                    credential_source: source.to_vec(),
                    offset,
                })?)
                .await
                .map_err(|_| Error::Authentication)?
                .into_inner();
            let content = zeroize::Zeroizing::new(piece.content);
            if piece.offset != offset
                || piece.descriptor.as_ref() != Some(&descriptor)
                || content.is_empty()
                || content.len() > 32 * 1024
                || bytes.len() + content.len() > descriptor.total_bytes as usize
            {
                return Err(Error::Authentication);
            }
            bytes.extend_from_slice(&content);
        }
        let secret =
            ExternalSecret::new(std::mem::take(&mut *bytes)).map_err(|_| Error::Authentication)?;
        let credentials = match c::ExecutionCredentialKind::try_from(descriptor.kind) {
            Ok(c::ExecutionCredentialKind::ApiKey) => NativeCredentials::api_key(secret)?,
            Ok(c::ExecutionCredentialKind::ChatgptTokens) => {
                NativeCredentials::external_token_file(
                    secret,
                    descriptor
                        .account_id
                        .as_deref()
                        .ok_or(Error::Authentication)?,
                )?
            }
            _ => return Err(Error::Authentication),
        };
        let binding = NativeAuthenticationBinding {
            agent_id: self.target.agent_id.clone(),
            conversation_id: self.target.conversation_id.clone(),
            operation_id: applied.operation_id,
            execution_id: applied.execution_id,
            authentication_id: applied.authentication_id,
            source,
        };
        self.check(&binding).await?;
        Ok(Some(NativeAuthentication {
            binding,
            credentials,
        }))
    }
}
impl NativeAuthenticationProvider for Authentication {
    fn acquire(&self) -> NativeAuthenticationFuture<'_, NativeAuthentication> {
        Box::pin(self.acquire_once())
    }
    fn check<'a>(
        &'a self,
        binding: &'a NativeAuthenticationBinding,
    ) -> NativeAuthenticationFuture<'a, ()> {
        Box::pin(async move {
            if binding.operation_id != self.target.operation_id
                || binding.agent_id != self.target.agent_id
                || binding.conversation_id != self.target.conversation_id
            {
                return Err(Error::Authentication);
            }
            self.client()
                .check_authentication(self.request(c::AuthenticationHandoffCheck {
                    target: Some(self.target.clone()),
                    credential_source: binding.source.to_vec(),
                })?)
                .await
                .map_err(|_| Error::Authentication)?;
            Ok(())
        })
    }
}
