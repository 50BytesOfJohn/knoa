//! Which servers Cloud mode talks to. Debug builds use staging, which `KNOA_API_URL`,
//! `KNOA_CLERK_ISSUER` and `KNOA_CLERK_CLIENT_ID` can override. Release builds use
//! production, and only when the build was given its OAuth client id.

use knoa_sync::Config;
use url::Url;

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
    pub fn load() -> Option<CloudConfig> {
        if cfg!(debug_assertions) {
            staging()
                .inspect_err(|err| tracing::error!("cloud configuration is invalid: {err}"))
                .ok()
        } else {
            production()
        }
    }

    /// The key the refresh token is stored under in the credentials.
    pub fn credential_url(&self) -> &str {
        self.sync.issuer.as_str()
    }
}

fn staging() -> Result<CloudConfig, url::ParseError> {
    let var = |name: &str, default: &str| std::env::var(name).unwrap_or_else(|_| default.into());
    Ok(CloudConfig {
        sync: Config {
            api: Url::parse(&var(
                "KNOA_API_URL",
                "https://knoa-api-staging.mynthio.workers.dev",
            ))?,
            issuer: Url::parse(&var(
                "KNOA_CLERK_ISSUER",
                "https://more-buck-9676.clerk.accounts.dev",
            ))?,
            client_id: var("KNOA_CLERK_CLIENT_ID", "C8fhAQh3MI0bMg9O"),
        },
        pricing: Url::parse("http://localhost:3001/pricing")?,
        web_app: Url::parse("http://localhost:3001/app")?,
    })
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
