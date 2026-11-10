//! Pages Knoa can't read: private hosts, and login, consent or bot walls.
//!
//! The AI must be told, or it describes "the PostHog login page" instead of inferring the
//! bookmark from its URL, browser title and folder.

use std::net::{Ipv4Addr, Ipv6Addr};

use url::{Host, Url};

const PRIVATE_SUFFIXES: [&str; 3] = [".local", ".internal", ".ts.net"];

/// A host on this machine or a private network: never fetched.
pub fn is_private(url: &Url) -> bool {
    match url.host() {
        None => true,
        Some(Host::Domain(host)) => {
            // `localhost.` is still localhost: a trailing dot only marks the name as absolute.
            let host = host.trim_end_matches('.');
            host == "localhost" || PRIVATE_SUFFIXES.iter().any(|suffix| host.ends_with(suffix))
        }
        Some(Host::Ipv4(ip)) => is_private_v4(ip),
        Some(Host::Ipv6(ip)) => is_private_v6(ip),
    }
}

fn is_private_v4(ip: Ipv4Addr) -> bool {
    // 100.64.0.0/10 is carrier-grade NAT, which Tailscale uses.
    let [first, second, ..] = ip.octets();
    let tailscale = first == 100 && (64..128).contains(&second);
    ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified() || tailscale
}

fn is_private_v6(ip: Ipv6Addr) -> bool {
    // `::ffff:192.168.1.4` reaches the IPv4 host, so it is as private as that host.
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_private_v4(v4);
    }
    ip.is_loopback() || ip.is_unspecified() || ip.is_unique_local() || ip.is_unicast_link_local()
}

const WALL_PATHS: &[&str] = &[
    "login",
    "signin",
    "sign-in",
    "sign_in",
    "auth",
    "sso",
    "consent",
    "challenge",
    "captcha",
];
const WALL_TITLES: &[&str] = &[
    "sign in",
    "log in",
    "login",
    "just a moment",
    "access denied",
    "attention required",
    "verify you are human",
    "before you continue",
    "403 forbidden",
];

/// Why the fetched page is not the bookmarked content, if it isn't.
pub fn wall(
    requested: &Url,
    landed: &Url,
    title: Option<&str>,
    text: &str,
) -> Option<&'static str> {
    let redirected = landed.path() != requested.path() || landed.host_str() != requested.host_str();
    if redirected
        && WALL_PATHS
            .iter()
            .any(|w| landed.path().to_ascii_lowercase().contains(w))
    {
        return Some("redirected to a login or consent page");
    }
    let title = title.unwrap_or_default().to_ascii_lowercase();
    if WALL_TITLES.iter().any(|w| title.starts_with(w)) {
        return Some("the page shows a login or verification wall");
    }
    let lower = text.to_ascii_lowercase();
    if text.len() < 400
        && (lower.contains("enable javascript") || lower.contains("javascript is required"))
    {
        return Some("the page needs JavaScript and has no readable content");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_and_tailnet_hosts_are_not_fetched() {
        for url in [
            "http://localhost:3000/",
            "http://192.168.1.4/",
            "http://100.101.5.6/",
            "https://box.tail1.ts.net/",
            "http://0.0.0.0:8080/",
            "http://[::ffff:192.168.1.4]/",
            "http://[fe80::1]/",
        ] {
            assert!(is_private(&Url::parse(url).unwrap()), "{url}");
        }
        assert!(!is_private(&Url::parse("https://example.com/").unwrap()));
    }
}
