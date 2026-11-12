//! Which servers Cloud mode talks to. Dev builds use staging, which `KNOA_API_URL`,
//! `KNOA_CLERK_ISSUER` and `KNOA_CLERK_CLIENT_ID` can override. `KNOA_WEB_URL` sets
//! the web app (default <http://localhost:3210>). Release-channel builds use
//! production, and only when the build was given its OAuth client id.

use anyhow::anyhow;
use knoa_sync::Config;
use url::Url;

use crate::channel::Channel;

/// Everything Cloud mode needs to know about where the account lives.
#[derive(Debug, Clone)]
pub struct CloudConfig {
    pub sync: Config,
    /// Where a free account goes to subscribe.
    pub pricing: Url,
    /// The web app, where subscribers manage their subscription.
    pub web_app: Url,
}

impl CloudConfig {
    /// `None` when this build has no cloud to sign in to.
    pub fn load() -> anyhow::Result<Option<CloudConfig>> {
        if Channel::current() == Channel::Dev {
            staging().map(Some)
        } else {
            Ok(production())
        }
    }

    /// The key the refresh token is stored under in the credentials.
    pub fn credential_url(&self) -> &str {
        self.sync.issuer.as_str()
    }
}

fn staging() -> anyhow::Result<CloudConfig> {
    let web = var("KNOA_WEB_URL", "http://localhost:3210");
    let web = web.trim_end_matches('/');
    Ok(CloudConfig {
        sync: Config {
            api: parse_var(
                "KNOA_API_URL",
                "https://knoa-api-staging.mynthio.workers.dev",
            )?,
            issuer: parse_var(
                "KNOA_CLERK_ISSUER",
                "https://more-buck-9676.clerk.accounts.dev",
            )?,
            client_id: var("KNOA_CLERK_CLIENT_ID", "C8fhAQh3MI0bMg9O"),
        },
        pricing: parse_named("KNOA_WEB_URL", &format!("{web}/pricing"))?,
        web_app: parse_named("KNOA_WEB_URL", &format!("{web}/app"))?,
    })
}

fn var(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.into())
}

fn parse_var(name: &str, default: &str) -> anyhow::Result<Url> {
    parse_named(name, &var(name, default))
}

/// Names the variable in the error, since the parser's own message doesn't say which.
fn parse_named(name: &str, value: &str) -> anyhow::Result<Url> {
    Url::parse(value).map_err(|err| anyhow!("{name} is {value:?}, which isn't a URL: {err}"))
}

/// The client id comes from the build, so a fork's builds don't sign in as Knoa.
fn production() -> Option<CloudConfig> {
    let client_id = option_env!("KNOA_CLERK_CLIENT_ID")?;
    Some(CloudConfig {
        sync: Config {
            api: Url::parse("https://api.useknoa.app").ok()?,
            issuer: Url::parse("https://clerk.useknoa.app").ok()?,
            client_id: client_id.to_owned(),
        },
        pricing: Url::parse("https://useknoa.app/pricing").ok()?,
        web_app: Url::parse("https://useknoa.app/app").ok()?,
    })
}
