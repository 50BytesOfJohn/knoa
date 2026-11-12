//! Clerk's token, revoke and userinfo endpoints, for a public client: the client id goes
//! in the form body and there is no secret.

use std::time::{Duration, Instant};

use reqwest::{StatusCode, header::CONTENT_TYPE};
use serde::Deserialize;
use url::form_urlencoded;

use crate::{Config, Error, Result};

/// What the token endpoint hands out.
#[derive(Clone)]
pub struct Tokens {
    pub access: String,
    /// Sent on sign-in, and again whenever Clerk rotates it.
    pub refresh: Option<String>,
    pub expires_at: Instant,
}

impl std::fmt::Debug for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tokens")
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    /// Seconds. Clerk's access tokens last a day.
    expires_in: Option<u64>,
}

#[derive(Deserialize)]
struct OAuthError {
    error: String,
    #[serde(default)]
    error_description: String,
}

pub(super) async fn exchange(
    http: &reqwest::Client,
    config: &Config,
    code: &str,
    redirect_uri: &str,
    verifier: &str,
) -> Result<Tokens> {
    let form = [
        ("grant_type", "authorization_code"),
        ("client_id", &config.client_id),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", verifier),
    ];
    match request_tokens(http, config, &form).await {
        Err(Error::SignedOut) => Err(Error::SignIn("Clerk refused the sign-in code".into())),
        other => other,
    }
}

/// New tokens from the refresh token. [`Error::SignedOut`] when Clerk no longer takes it.
pub(crate) async fn refresh(
    http: &reqwest::Client,
    config: &Config,
    refresh_token: &str,
) -> Result<Tokens> {
    let form = [
        ("grant_type", "refresh_token"),
        ("client_id", &config.client_id),
        ("refresh_token", refresh_token),
    ];
    request_tokens(http, config, &form).await
}

/// Revokes the refresh token, so it can't be used again. Access tokens are JWTs, which
/// can't be revoked; they run out within a day.
pub async fn revoke(http: &reqwest::Client, config: &Config, refresh_token: &str) -> Result<()> {
    let form = [
        ("token", refresh_token),
        ("token_type_hint", "refresh_token"),
        ("client_id", &config.client_id),
    ];
    let response = post_form(http, config, "oauth/token/revoke", &form).await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(oauth_error(response).await)
    }
}

/// The signed-in user's email, from Clerk's userinfo endpoint.
pub async fn user_email(
    http: &reqwest::Client,
    config: &Config,
    access_token: &str,
) -> Result<Option<String>> {
    #[derive(Deserialize)]
    struct UserInfo {
        email: Option<String>,
    }
    let url = endpoint(config, "oauth/userinfo")?;
    let response = http.get(url).bearer_auth(access_token).send().await?;
    if !response.status().is_success() {
        return Err(oauth_error(response).await);
    }
    Ok(response.json::<UserInfo>().await?.email)
}

async fn request_tokens(
    http: &reqwest::Client,
    config: &Config,
    form: &[(&str, &str)],
) -> Result<Tokens> {
    let response = post_form(http, config, "oauth/token", form).await?;
    if !response.status().is_success() {
        return Err(oauth_error(response).await);
    }
    let body: TokenResponse = response.json().await?;
    Ok(Tokens {
        access: body.access_token,
        refresh: body.refresh_token.filter(|token| !token.is_empty()),
        expires_at: Instant::now() + Duration::from_secs(body.expires_in.unwrap_or(3600)),
    })
}

async fn post_form(
    http: &reqwest::Client,
    config: &Config,
    path: &str,
    form: &[(&str, &str)],
) -> Result<reqwest::Response> {
    let body = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(form)
        .finish();
    Ok(http
        .post(endpoint(config, path)?)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?)
}

fn endpoint(config: &Config, path: &str) -> Result<url::Url> {
    config
        .issuer
        .join(path)
        .map_err(|err| Error::Config(format!("bad issuer URL: {err}")))
}

/// `invalid_grant` and `invalid_client` mean the grant is gone for good; the rest of
/// Clerk's answers are reported as they are.
async fn oauth_error(response: reqwest::Response) -> Error {
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    let parsed: Option<OAuthError> = serde_json::from_str(&text).ok();
    match parsed {
        Some(err) if status == StatusCode::BAD_REQUEST || status == StatusCode::UNAUTHORIZED => {
            if matches!(err.error.as_str(), "invalid_grant" | "invalid_client") {
                Error::SignedOut
            } else {
                server_error(status, err.error, err.error_description)
            }
        }
        Some(err) => server_error(status, err.error, err.error_description),
        None => server_error(status, String::new(), text),
    }
}

fn server_error(status: StatusCode, code: String, message: String) -> Error {
    Error::Server {
        status: status.as_u16(),
        code,
        message,
    }
}
