//! Errors from AI providers, sorted by what the caller should do about them.

use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the API key was rejected")]
    Unauthorized,
    #[error("the account is out of credits")]
    NoCredits,
    #[error("rate limited")]
    RateLimited { retry_after: Option<Duration> },
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("the provider failed ({status}): {message}")]
    Provider { status: u16, message: String },
    #[error("the model's answer was unusable: {0}")]
    BadAnswer(String),
}

impl Error {
    /// Worth retrying the same request later.
    pub fn is_transient(&self) -> bool {
        match self {
            Error::RateLimited { .. } | Error::Network(_) => true,
            Error::Provider { status, .. } => *status >= 500 || *status == 408,
            _ => false,
        }
    }

    /// The setup is wrong, so every request will fail until the user fixes it.
    pub fn needs_user(&self) -> bool {
        matches!(self, Error::Unauthorized | Error::NoCredits)
    }

    /// Another model might do better: the answer was malformed, or no endpoint serves the
    /// request (OpenRouter's 404 when a model lacks a required parameter).
    pub fn try_other_model(&self) -> bool {
        matches!(self, Error::BadAnswer(_)) || self.is_refusal()
    }

    /// The endpoint refused the request itself, so it wasn't billed: a model that can't
    /// take an image, or lacks a parameter the request requires.
    pub fn is_refusal(&self) -> bool {
        matches!(
            self,
            Error::Provider {
                status: 400 | 404 | 422,
                ..
            }
        )
    }
}

/// The error a chain of fallback models reports when every one failed. A bad answer, which
/// was billed, outranks a refusal, so the chain only reads as refused when no model took
/// the request at all.
#[derive(Debug, Default)]
pub(crate) struct Failures(Option<Error>);

impl Failures {
    pub(crate) fn add(&mut self, err: Error) {
        let billed = |err: &Error| matches!(err, Error::BadAnswer(_));
        if !self.0.as_ref().is_some_and(billed) || billed(&err) {
            self.0 = Some(err);
        }
    }

    pub(crate) fn into_error(self) -> Error {
        self.0
            .unwrap_or_else(|| Error::BadAnswer("no chat model configured".into()))
    }
}
