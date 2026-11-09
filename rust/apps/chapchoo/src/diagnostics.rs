//! Application-owned diagnostics exclude dependency protocol payloads.
use tracing_subscriber::{filter::filter_fn, prelude::*};
fn allowed(target: &str) -> bool {
    // SDK TRACE serializes login metadata. EnvFilter specificity alone cannot
    // be a security boundary because operators can enable narrower targets.
    !target.starts_with("agent_client_protocol")
}
pub(super) fn initialize() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("chapchoo=info")),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_filter(filter_fn(|metadata| allowed(metadata.target()))),
        )
        .init();
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    #[derive(Clone)]
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .map_err(|_| std::io::Error::other("Capture unavailable"))?
                .extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn enabled_trace_cannot_publish_sdk_login_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let writer = Capture(bytes.clone());
        let subscriber = tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new("trace"))
            .with(
                tracing_subscriber::fmt::layer()
                    .without_time()
                    .with_ansi(false)
                    .with_writer(move || writer.clone())
                    .with_filter(filter_fn(|metadata| allowed(metadata.target()))),
            );
        tracing::subscriber::with_default(subscriber, || {
            tracing::trace!(target:"agent_client_protocol::jsonrpc::transport_actor",message="synthetic-api-key-must-not-appear");
            tracing::info!(target:"chapchoo::connector",stage="authentication","Native operation rejected");
        });
        let output = String::from_utf8(bytes.lock().map_err(|_| "Capture unavailable")?.clone())?;
        assert!(!output.contains("synthetic-api-key"));
        assert!(output.contains("Native operation rejected"));
        Ok(())
    }
}
