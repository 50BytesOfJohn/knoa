//! Pages Knoa can't read: private hosts, and login, consent or bot walls.
//!
//! The AI must be told, or it describes "the PostHog login page" instead of inferring the
//! bookmark from its URL, browser title and folder.

use std::net::IpAddr;

use url::Url;

const PRIVATE_SUFFIXES: [&str; 3] = [".local", ".internal", ".ts.net"];

/// A host on this machine or a private network: never fetched.
pub fn is_private(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return true;
    };
    if host == "localhost" || PRIVATE_SUFFIXES.iter().any(|suffix| host.ends_with(suffix)) {
        return true;
    }
    match host.trim_matches(['[', ']']).parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => {
            // 100.64.0.0/10 is carrier-grade NAT, which Tailscale uses.
            let tailscale = ip.octets()[0] == 100 && (64..128).contains(&ip.octets()[1]);
            ip.is_private() || ip.is_loopback() || ip.is_link_local() || tailscale
        }
        Ok(IpAddr::V6(ip)) => ip.is_loopback() || ip.is_unique_local(),
        Err(_) => false,
    }
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
        ] {
            assert!(is_private(&Url::parse(url).unwrap()), "{url}");
        }
        assert!(!is_private(&Url::parse("https://example.com/").unwrap()));
    }
}
