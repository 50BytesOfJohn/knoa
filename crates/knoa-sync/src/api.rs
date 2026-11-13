//! The Knoa API's endpoints. Every request carries a fresh access token; one the API
//! refuses is replaced and the request sent once more.

use std::sync::Arc;

use knoa_core::protocol::{
    Account, ErrorResponse, ImageResponse, MAX_PULL, McpSettings, McpSettingsResponse, Mutation,
    OrganizeRequest, OrganizeResponse, PullResponse, PushRequest, PushResponse,
};
use reqwest::{RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use url::Url;

use crate::{Error, Result, Session};

pub struct Api {
    session: Arc<Session>,
}

impl Api {
    pub fn new(session: Arc<Session>) -> Api {
        Api { session }
    }

    pub fn session(&self) -> &Arc<Session> {
        &self.session
    }

    pub async fn me(&self) -> Result<Account> {
        let url = self.url("v1/me")?;
        self.json(|http, token| http.get(url.clone()).bearer_auth(token))
            .await
    }

    /// The access granted to AI clients, and the URL used to connect them.
    pub async fn mcp_settings(&self) -> Result<McpSettingsResponse> {
        let url = self.url("v1/mcp/settings")?;
        self.json(|http, token| http.get(url.clone()).bearer_auth(token))
            .await
    }

    /// Replaces the default access and all workspace exceptions.
    pub async fn set_mcp_settings(&self, settings: &McpSettings) -> Result<McpSettingsResponse> {
        let url = self.url("v1/mcp/settings")?;
        self.json(|http, token| http.put(url.clone()).bearer_auth(token).json(settings))
            .await
    }

    /// At most `MAX_PUSH` mutations.
    pub async fn push(&self, mutations: Vec<Mutation>) -> Result<PushResponse> {
        let url = self.url("v1/sync/push")?;
        let body = PushRequest { mutations };
        self.json(|http, token| http.post(url.clone()).bearer_auth(token).json(&body))
            .await
    }

    /// The changes after `cursor`, with page text, which the desktop indexes for search.
    pub async fn pull(&self, cursor: i64) -> Result<PullResponse> {
        self.pull_page(cursor, true).await
    }

    /// The changes after `cursor` without page text, for a look at the whole account that
    /// stays small.
    pub async fn pull_without_texts(&self, cursor: i64) -> Result<PullResponse> {
        self.pull_page(cursor, false).await
    }

    async fn pull_page(&self, cursor: i64, texts: bool) -> Result<PullResponse> {
        let mut url = self.url("v1/sync/pull")?;
        url.query_pairs_mut()
            .append_pair("cursor", &cursor.to_string())
            .append_pair("limit", &MAX_PULL.to_string())
            .append_pair("texts", if texts { "1" } else { "0" });
        self.json(|http, token| http.get(url.clone()).bearer_auth(token))
            .await
    }

    /// Asks the cloud organizer to run again. Subscribers only.
    pub async fn organize(&self, request: &OrganizeRequest) -> Result<OrganizeResponse> {
        let url = self.url("v1/organize")?;
        self.json(|http, token| http.post(url.clone()).bearer_auth(token).json(request))
            .await
    }

    /// Uploads a bookmark's thumbnail; the server records its key on the bookmark.
    pub async fn upload_image(
        &self,
        bookmark: &str,
        bytes: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<ImageResponse> {
        let url = self.url(&format!("v1/images/{bookmark}"))?;
        self.json(|http, token| {
            http.put(url.clone())
                .bearer_auth(token)
                .header("x-image-width", width)
                .header("x-image-height", height)
                .body(bytes.clone())
        })
        .await
    }

    /// Where a thumbnail the server keeps can be downloaded, without signing in.
    pub fn image_url(&self, image_key: &str) -> Result<Url> {
        self.url(&format!("img/{image_key}"))
    }

    pub(crate) fn socket_url(&self) -> Result<Url> {
        let mut url = self.url("v1/sync/socket")?;
        let scheme = if url.scheme() == "http" { "ws" } else { "wss" };
        url.set_scheme(scheme)
            .map_err(|()| Error::Config("the API URL can't take a socket".into()))?;
        Ok(url)
    }

    fn url(&self, path: &str) -> Result<Url> {
        self.session
            .config()
            .api
            .join(path)
            .map_err(|err| Error::Config(format!("bad API URL: {err}")))
    }

    async fn json<T: DeserializeOwned>(
        &self,
        build: impl Fn(&reqwest::Client, &str) -> RequestBuilder,
    ) -> Result<T> {
        let response = self.send(build).await?;
        Ok(response.json().await?)
    }

    async fn send(
        &self,
        build: impl Fn(&reqwest::Client, &str) -> RequestBuilder,
    ) -> Result<Response> {
        let http = self.session.http();
        let token = self.session.access_token().await?;
        let mut response = build(http, &token).send().await?;
        if response.status() == StatusCode::UNAUTHORIZED {
            self.session.invalidate(&token).await;
            let token = self.session.access_token().await?;
            response = build(http, &token).send().await?;
        }
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(api_error(response).await)
        }
    }
}

async fn api_error(response: Response) -> Error {
    let status = response.status().as_u16();
    let text = response.text().await.unwrap_or_default();
    match serde_json::from_str::<ErrorResponse>(&text) {
        Ok(body) => Error::Server {
            status,
            code: body.error,
            message: body.message,
        },
        Err(_) => Error::Server {
            status,
            code: String::new(),
            message: text.chars().take(200).collect(),
        },
    }
}
