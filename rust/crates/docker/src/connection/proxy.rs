//! Detect selected Go/Docker TCP proxy policy before opening a direct stream.
//! Proxy transports are not implemented; an effective proxy is a compatibility
//! error rather than permission to bypass the account's routing policy.
use super::*;
use std::net::IpAddr;
fn variable(upper: &str, lower: &str) -> Result<Option<String>, Error> {
    for name in [upper, lower] {
        if let Some(value) = std::env::var_os(name).filter(|value| !value.is_empty()) {
            return value
                .into_string()
                .map(Some)
                .map_err(|_| Error::Configuration);
        }
    }
    Ok(None)
}
fn port_pattern(value: &str) -> (&str, Option<&str>) {
    if let Some(value) = value.strip_prefix('[')
        && let Some((host, port)) = value.split_once("]:")
    {
        return (host, Some(port));
    }
    if value.bytes().filter(|b| *b == b':').count() == 1
        && let Some((host, port)) = value.rsplit_once(':')
    {
        return (host, Some(port));
    }
    (value, None)
}
fn in_network(host: IpAddr, network: &str) -> bool {
    let Some((address, prefix)) = network.split_once('/') else {
        return false;
    };
    let Ok(address) = address.parse::<IpAddr>() else {
        return false;
    };
    let Ok(prefix) = prefix.parse::<u32>() else {
        return false;
    };
    match (host, address) {
        (IpAddr::V4(host), IpAddr::V4(address)) if prefix <= 32 => {
            let mask = u32::MAX.checked_shl(32 - prefix).unwrap_or(0);
            u32::from(host) & mask == u32::from(address) & mask
        }
        (IpAddr::V6(host), IpAddr::V6(address)) if prefix <= 128 => {
            let mask = u128::MAX.checked_shl(128 - prefix).unwrap_or(0);
            u128::from(host) & mask == u128::from(address) & mask
        }
        _ => false,
    }
}
fn bypass(host: &str, port: u16, no_proxy: &str) -> bool {
    let host = host.to_ascii_lowercase();
    let ip = host.parse::<IpAddr>().ok();
    // Go's ProxyFromEnvironment always bypasses loopback and localhost.
    if host == "localhost" || ip.is_some_and(|ip| ip.is_loopback()) {
        return true;
    }
    no_proxy.split(',').any(|pattern| {
        let pattern = pattern.trim().to_ascii_lowercase();
        if pattern == "*" {
            return true;
        }
        if let Some(ip) = ip
            && in_network(ip, &pattern)
        {
            return true;
        }
        let (pattern, expected_port) = port_pattern(&pattern);
        if expected_port.is_some_and(|p| p.parse::<u16>().ok() != Some(port)) {
            return false;
        }
        if let Ok(pattern) = pattern.trim_matches(['[', ']']).parse::<IpAddr>() {
            return ip == Some(pattern);
        }
        if ip.is_some() || pattern.is_empty() {
            return false;
        }
        let pattern = pattern
            .strip_prefix("*.")
            .map(|p| format!(".{p}"))
            .unwrap_or_else(|| pattern.to_owned());
        if pattern.starts_with('.') {
            host.ends_with(&pattern)
        } else {
            host == pattern || host.ends_with(&format!(".{pattern}"))
        }
    })
}
pub(super) fn check(selected: Selection) -> Result<Selection, Error> {
    if let Endpoint::Tcp { host, port } = &selected.endpoint {
        let proxy = if selected.tls.is_some() {
            variable("HTTPS_PROXY", "https_proxy")?
        } else {
            variable("HTTP_PROXY", "http_proxy")?
        };
        if proxy.is_some()
            && !bypass(
                host,
                *port,
                &variable("NO_PROXY", "no_proxy")?.unwrap_or_default(),
            )
        {
            return Err(Error::UnsupportedProxy);
        }
    }
    Ok(selected)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn docker_proxy_bypass_is_scoped_to_the_selected_host_port_or_network() {
        for (host, port, no_proxy, expected) in [
            ("localhost", 2375, "", true),
            ("127.0.0.2", 2375, "", true),
            ("::1", 2376, "", true),
            ("daemon.example", 2375, "", false),
            ("daemon.example", 2375, "*", true),
            ("daemon.example", 2375, "example", true),
            ("example", 2375, ".example", false),
            ("daemon.example", 2375, "*.example", true),
            ("notexample", 2375, "example", false),
            ("daemon.example", 2375, "daemon.example:2376", false),
            ("daemon.example", 2376, "daemon.example:2376", true),
            ("10.2.3.4", 2375, "10.0.0.0/8", true),
            ("11.2.3.4", 2375, "10.0.0.0/8", false),
            ("2001:db8::2", 2376, "[2001:db8::2]:2376", true),
            ("2001:db8::2", 2375, "[2001:db8::2]:2376", false),
            ("2001:db8::2", 2376, "2001:db8::/32", true),
        ] {
            assert_eq!(bypass(host, port, no_proxy), expected, "{host}:{port}");
        }
    }
}
