//! OpenRouter: a thin hand-written client for the three endpoints Knoa uses (ADR 0003).

mod chat;
mod decisions;

use std::time::Duration;

use reqwest::{Client, Response, StatusCode, header};
use serde::{Deserialize, de::DeserializeOwned};

use crate::Error;

const BASE_URL: &str = "https://openrouter.ai";

pub struct OpenRouter {
    client: Client,
    api_key: String,
    chat_models: Vec<String>,
    classifier_model: Option<String>,
}

impl OpenRouter {
    pub fn new(
        api_key: String,
        chat_models: Vec<String>,
        classifier_model: Option<String>,
    ) -> Result<OpenRouter, Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .default_headers(
                [
                    // OpenRouter's app attribution headers.
                    (
                        header::REFERER,
                        header::HeaderValue::from_static("https://github.com/50BytesOfJohn/knoa"),
                    ),
                    (
                        header::HeaderName::from_static("x-title"),
                        header::HeaderValue::from_static("Knoa"),
                    ),
                ]
                .into_iter()
                .collect(),
            )
            .build()?;
        Ok(OpenRouter {
            client,
            api_key,
            chat_models,
            classifier_model,
        })
    }

    pub fn has_classifier(&self) -> bool {
        self.classifier_model.is_some()
    }

    /// Checks that the key works, without spending credits.
    pub async fn check_key(&self) -> Result<(), Error> {
        let response = self
            .client
            .get(format!("{BASE_URL}/api/v1/key"))
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        read_json::<serde_json::Value>(response).await.map(drop)
    }

    async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<T, Error> {
        let response = self
            .client
            .post(format!("{BASE_URL}{path}"))
            .bearer_auth(&self.api_key)
            .json(body)
            .send()
            .await?;
        read_json(response).await
    }
}

#[derive(Deserialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(Deserialize)]
struct ErrorDetail {
    message: String,
}

async fn read_json<T: DeserializeOwned>(response: Response) -> Result<T, Error> {
    let status = response.status();
    let retry_after = response
        .headers()
        .get(header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok()?.parse().ok())
        .map(Duration::from_secs);
    let body = response.bytes().await?;
    if status.is_success() {
        return serde_json::from_slice(&body)
            .map_err(|err| Error::BadAnswer(format!("unexpected response: {err}")));
    }
    let message = serde_json::from_slice::<ErrorBody>(&body).map_or_else(
        |_| String::from_utf8_lossy(&body).chars().take(300).collect(),
        |b| b.error.message,
    );
    Err(match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Error::Unauthorized,
        StatusCode::PAYMENT_REQUIRED => Error::NoCredits,
        StatusCode::TOO_MANY_REQUESTS => Error::RateLimited { retry_after },
        _ => Error::Provider {
            status: status.as_u16(),
            message,
        },
    })
}
