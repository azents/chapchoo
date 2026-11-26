use super::McpDiscoveryFailure;
use chapchoo_persistence::McpTestTarget;
use reqwest::{
    Client,
    dns::{Addrs, Name, Resolve, Resolving},
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use rustls_platform_verifier::BuilderVerifierExt;
use std::{
    io,
    net::{IpAddr, Ipv4Addr, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Semaphore;
use url::Host;
use zeroize::Zeroizing;

#[derive(Clone, Debug)]
pub(super) struct OutboundMcpHttp {
    dns: Arc<PublicDns>,
}

impl OutboundMcpHttp {
    pub(super) fn new() -> Self {
        Self {
            dns: Arc::new(PublicDns {
                slots: Arc::new(Semaphore::new(16)),
            }),
        }
    }

    pub(super) fn client(&self, target: &McpTestTarget) -> Result<Client, McpDiscoveryFailure> {
        let parsed = target.endpoint.url();
        match parsed.host() {
            // The connector does not need DNS for a literal. Enforce the same policy
            // here before handing over the only URL this client will use.
            Some(Host::Ipv4(addr)) if !is_public(addr.into()) => {
                return Err(McpDiscoveryFailure::DestinationDenied);
            }
            Some(Host::Ipv6(addr)) if !is_public(addr.into()) => {
                return Err(McpDiscoveryFailure::DestinationDenied);
            }
            None => return Err(McpDiscoveryFailure::DestinationDenied),
            _ => {}
        }
        let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .map_err(|_| McpDiscoveryFailure::Unavailable)?
        .with_platform_verifier()
        .map_err(|_| McpDiscoveryFailure::Unavailable)?
        .with_no_client_auth();
        self.builder(tls, target)?
            .build()
            .map_err(|_| McpDiscoveryFailure::Unavailable)
    }

    pub(super) fn builder(
        &self,
        tls: rustls::ClientConfig,
        target: &McpTestTarget,
    ) -> Result<reqwest::ClientBuilder, McpDiscoveryFailure> {
        let mut headers = HeaderMap::new();
        if let Some(secret) = &target.bearer {
            let mut bytes = Zeroizing::new(b"Bearer ".to_vec());
            bytes.extend_from_slice(secret.as_bytes());
            let mut value =
                HeaderValue::from_bytes(&bytes).map_err(|_| McpDiscoveryFailure::Authentication)?;
            value.set_sensitive(true);
            headers.insert(AUTHORIZATION, value);
        }
        Ok(Client::builder()
            .tls_backend_preconfigured(tls)
            .default_headers(headers)
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .dns_resolver(self.dns.clone())
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(8))
            .pool_max_idle_per_host(1))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("MCP destination did not resolve exclusively to public addresses")]
pub(super) struct NonPublicDestination;

#[derive(Debug)]
struct PublicDns {
    slots: Arc<Semaphore>,
}

impl Resolve for PublicDns {
    fn resolve(&self, name: Name) -> Resolving {
        let slots = self.slots.clone();
        // The fully qualified name must not use the installation's DNS search suffix.
        let fqdn = format!("{}.", name.as_str().trim_end_matches('.'));
        Box::pin(async move {
            let slot = slots
                .try_acquire_owned()
                .map_err(|_| io::Error::other("MCP DNS capacity exhausted"))?;
            let addrs = tokio::task::spawn_blocking(move || {
                // Keep the permit until the blocking lookup actually finishes.
                let _slot = slot;
                (fqdn.as_str(), 0).to_socket_addrs()
            })
            .await??;
            if addrs.len() == 0 || addrs.as_slice().iter().any(|addr| !is_public(addr.ip())) {
                return Err(NonPublicDestination.into());
            }
            // This result is passed straight to hyper's dialer; no second lookup
            // occurs between address validation and dialing (or on address fallback).
            Ok(Box::new(addrs) as Addrs)
        })
    }
}

fn is_public(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(addr) => is_public_v4(addr),
        // Only global-unicast space, excluding special-use and tunnel prefixes.
        // This rejects mapped/compatible, NAT64, ULA, link-local and multicast.
        IpAddr::V6(addr) => {
            let [a, b, ..] = addr.segments();
            a & 0xe000 == 0x2000
                && !(a == 0x2001 && (b <= 0x01ff || b == 0x0db8))
                && a != 0x2002
                && a != 0x3ffe
                && !(a == 0x3fff && b < 0x1000)
        }
    }
}

fn is_public_v4(addr: Ipv4Addr) -> bool {
    let [a, b, c, _] = addr.octets();
    !matches!(
        (a, b, c),
        (0 | 10 | 127 | 224..=255, _, _)
            | (100, 64..=127, _)
            | (169, 254, _)
            | (172, 16..=31, _)
            | (192, 0, 0 | 2)
            | (192, 88, 99)
            | (192, 168, _)
            | (198, 18 | 19, _)
            | (198, 51, 100)
            | (203, 0, 113)
    )
}
