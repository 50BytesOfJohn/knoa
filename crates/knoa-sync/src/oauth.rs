//! Signing in with Clerk as an OAuth provider: authorization code with PKCE, delivered to
//! a one-shot listener on the loopback interface (RFC 8252), then refresh and revoke.

mod loopback;
mod pkce;
mod token;

use std::time::Duration;

use url::Url;

use self::{loopback::Loopback, pkce::Pkce};
use crate::{Config, Error, Result};
pub(crate) use token::refresh;
pub use token::{Tokens, revoke, user_email};

/// `offline_access` is what gets a refresh token, so the app stays signed in.
const SCOPES: &str = "profile email offline_access";
/// Long enough to sign up from scratch, short enough not to hold a port open all day.
const SIGN_IN_TIMEOUT: Duration = Duration::from_mins(5);

/// A sign-in waiting for the browser. Dropping it stops listening.
pub struct SignIn {
    config: Config,
    loopback: Loopback,
    pkce: Pkce,
    state: String,
    url: Url,
}

impl SignIn {
    /// Starts listening for the browser's redirect and builds the page to open.
    pub async fn start(config: &Config) -> Result<SignIn> {
        let loopback = Loopback::bind().await?;
        let pkce = Pkce::new()?;
        let state = pkce::random_token()?;
        let mut url = config.issuer.join("oauth/authorize").map_err(bad_url)?;
        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &config.client_id)
            .append_pair("redirect_uri", &loopback.redirect_uri())
            .append_pair("scope", SCOPES)
            .append_pair("state", &state)
            .append_pair("code_challenge", &pkce.challenge)
            .append_pair("code_challenge_method", "S256");
        Ok(SignIn {
            config: config.clone(),
            loopback,
            pkce,
            state,
            url,
        })
    }

    /// The page that signs the user in, for the system browser.
    pub fn authorize_url(&self) -> &Url {
        &self.url
    }

    /// Waits for the browser to come back, then trades the code for tokens.
    pub async fn finish(self, http: &reqwest::Client) -> Result<Tokens> {
        let redirect_uri = self.loopback.redirect_uri();
        let code = tokio::time::timeout(SIGN_IN_TIMEOUT, self.loopback.wait(&self.state))
            .await
            .map_err(|_| Error::TimedOut)??;
        let tokens = token::exchange(
            http,
            &self.config,
            &code,
            &redirect_uri,
            &self.pkce.verifier,
        )
        .await?;
        if tokens.refresh.is_none() {
            return Err(Error::SignIn("Clerk sent no refresh token".into()));
        }
        Ok(tokens)
    }
}

fn bad_url(err: url::ParseError) -> Error {
    Error::Config(format!("bad issuer URL: {err}"))
}
