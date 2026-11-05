//! Choosing an AI provider from settings.
//!
//! Adding a provider means a new `ProviderKind`, a module implementing `LanguageModel`
//! (and `Classifier` if it has one) that records each answered request's usage in a
//! `Meter`, and a match arm in each `Provider` method.

use knoa_core::Usage;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    Choice, Classifier, Error, LanguageModel, Question, StructuredReply, StructuredRequest,
    openrouter::OpenRouter,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    OpenRouter,
}

impl ProviderKind {
    pub fn name(self) -> &'static str {
        match self {
            ProviderKind::OpenRouter => "OpenRouter",
        }
    }

    /// The stable id, as in the settings file and stored usage.
    pub fn id(self) -> &'static str {
        match self {
            ProviderKind::OpenRouter => "openrouter",
        }
    }

    /// Where the user sees the provider's own record of spending.
    pub fn activity_url(self) -> &'static str {
        match self {
            ProviderKind::OpenRouter => "https://openrouter.ai/activity",
        }
    }

    /// Where the user gets an API key.
    pub fn keys_url(self) -> &'static str {
        match self {
            ProviderKind::OpenRouter => "https://openrouter.ai/settings/keys",
        }
    }

    /// The credential store's account name for this provider's key.
    pub fn credential_url(self) -> &'static str {
        match self {
            ProviderKind::OpenRouter => "https://openrouter.ai",
        }
    }
}

/// The AI part of the settings file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    /// Chat models in fallback order.
    pub chat_models: Vec<String>,
    /// The classifier for kinds and folders. Without one, the chat model picks them.
    pub classifier_model: Option<String>,
}

impl ProviderConfig {
    /// The defaults chosen in the enrichment research: cheap, honest and fast.
    pub fn default_for(kind: ProviderKind) -> ProviderConfig {
        match kind {
            ProviderKind::OpenRouter => ProviderConfig {
                kind,
                chat_models: vec![
                    "openai/gpt-6-luna".into(),
                    "anthropic/claude-haiku-5.5".into(),
                ],
                classifier_model: Some("perplexity/pplx-decider-v1.2".into()),
            },
        }
    }
}

/// A configured provider, ready for requests.
pub enum Provider {
    OpenRouter(OpenRouter),
}

impl Provider {
    pub fn new(config: &ProviderConfig, api_key: String) -> Result<Provider, Error> {
        match config.kind {
            ProviderKind::OpenRouter => Ok(Provider::OpenRouter(OpenRouter::new(
                api_key,
                config.chat_models.clone(),
                config.classifier_model.clone(),
            )?)),
        }
    }

    /// Checks the key with a free request.
    pub async fn check_key(&self) -> Result<(), Error> {
        match self {
            Provider::OpenRouter(p) => p.check_key().await,
        }
    }

    pub fn classifier(&self) -> Option<&Self> {
        match self {
            Provider::OpenRouter(p) => p.has_classifier().then_some(self),
        }
    }

    pub fn kind(&self) -> ProviderKind {
        match self {
            Provider::OpenRouter(_) => ProviderKind::OpenRouter,
        }
    }

    /// What the requests made since the last call used. Requests that failed outright
    /// aren't billed, so they aren't here.
    pub fn take_usage(&self) -> Vec<Usage> {
        match self {
            Provider::OpenRouter(p) => p.meter.take(),
        }
    }
}

impl LanguageModel for Provider {
    async fn generate(&self, request: &StructuredRequest) -> Result<StructuredReply, Error> {
        match self {
            Provider::OpenRouter(p) => p.generate(request).await,
        }
    }
}

impl Classifier for Provider {
    async fn classify(&self, state: &Value, questions: &[Question]) -> Result<Vec<f32>, Error> {
        match self {
            Provider::OpenRouter(p) => p.classify(state, questions).await,
        }
    }

    async fn choose(&self, state: &Value, choice: &Choice) -> Result<Vec<f32>, Error> {
        match self {
            Provider::OpenRouter(p) => p.choose(state, choice).await,
        }
    }
}
