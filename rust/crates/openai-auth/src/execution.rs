//! Materialize the inspected native external-token schema without refresh authority.
use super::*;
impl chapchoo_service::NativeTokenEncoder for OpenAiDeviceClient {
    fn encode(
        &self,
        bundle: &ExternalSecret,
        account_id: &str,
        applied_at: i64,
    ) -> Result<ExternalSecret, chapchoo_service::NativeTokenEncodingFailure> {
        native_chatgpt_credentials(bundle, account_id, applied_at)
            .map_err(|_| chapchoo_service::NativeTokenEncodingFailure)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionCredentialError;
impl fmt::Display for ExecutionCredentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Selected credentials are not eligible for native token application")
    }
}
impl Error for ExecutionCredentialError {}
#[derive(Serialize)]
struct NativeFile<'a> {
    auth_mode: &'static str,
    #[serde(rename = "OPENAI_API_KEY")]
    api_key: Option<&'a str>,
    tokens: NativeTokens<'a>,
    last_refresh: String,
}
#[derive(Serialize)]
struct NativeTokens<'a> {
    // Native's external-token constructor serializes access-JWT-derived identity
    // as id_token, with explicit account binding and no usable refresh credential.
    id_token: &'a str,
    access_token: &'a str,
    refresh_token: &'static str,
    account_id: &'a str,
}
/// Caller separately revalidates current exact execution/applied-record authority.
/// This schema conversion is not signature verification or native adoption.
pub fn native_chatgpt_credentials(
    bundle: &ExternalSecret,
    account_id: &str,
    applied_at: i64,
) -> Result<ExternalSecret, ExecutionCredentialError> {
    if applied_at < 0 {
        return Err(ExecutionCredentialError);
    }
    let bundle: TokenBundle =
        serde_json::from_slice(bundle.expose()).map_err(|_| ExecutionCredentialError)?;
    required(account_id, 4096).map_err(|_| ExecutionCredentialError)?;
    required(&bundle.access_token, 65536).map_err(|_| ExecutionCredentialError)?;
    // Refuse incompatible/unknown account metadata instead of inventing another
    // account or passing an opaque bearer as a parseable native identity JWT.
    if metadata(Some(&bundle.access_token)).account_id.as_deref() != Some(account_id) {
        return Err(ExecutionCredentialError);
    }
    let file = NativeFile {
        auth_mode: "chatgptAuthTokens",
        api_key: None,
        tokens: NativeTokens {
            id_token: &bundle.access_token,
            access_token: &bundle.access_token,
            refresh_token: "",
            account_id,
        },
        // The native reader requires this field even for externally managed
        // access tokens. Use the retained application time so every bounded read
        // materializes the exact same bytes without granting refresh authority.
        last_refresh: jiff::Timestamp::from_millisecond(applied_at)
            .map_err(|_| ExecutionCredentialError)?
            .to_string(),
    };
    ExternalSecret::new(serde_json::to_vec(&file).map_err(|_| ExecutionCredentialError)?)
        .map_err(|_| ExecutionCredentialError)
}
