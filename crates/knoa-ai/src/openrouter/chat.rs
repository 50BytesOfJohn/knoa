//! Chat Completions with strict structured output.

use serde::Deserialize;
use serde_json::json;

use super::OpenRouter;
use crate::{Error, LanguageModel, StructuredReply, StructuredRequest};

#[derive(Deserialize)]
struct Completion {
    model: String,
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

impl LanguageModel for OpenRouter {
    async fn generate(&self, request: &StructuredRequest) -> Result<StructuredReply, Error> {
        let mut last_error = Error::BadAnswer("no chat model configured".into());
        for model in &self.chat_models {
            match self.complete(model, request).await {
                Ok(reply) => return Ok(reply),
                Err(err) if err.try_other_model() => {
                    tracing::debug!(model, "trying the next model: {err}");
                    last_error = err;
                }
                Err(err) => return Err(err),
            }
        }
        Err(last_error)
    }
}

impl OpenRouter {
    async fn complete(
        &self,
        model: &str,
        request: &StructuredRequest,
    ) -> Result<StructuredReply, Error> {
        let messages: Vec<_> = request
            .messages
            .iter()
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();
        let body = json!({
            "model": model,
            "messages": messages,
            "max_tokens": request.max_tokens,
            "response_format": {
                "type": "json_schema",
                "json_schema": { "name": request.schema_name, "strict": true, "schema": request.schema },
            },
            // Reasoning made answers slower and no better in the enrichment research.
            "reasoning": { "effort": "none" },
            // Only route to endpoints that honour the schema.
            "provider": { "require_parameters": true },
        });
        let completion: Completion = self.post("/api/v1/chat/completions", &body).await?;
        let choice = completion
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| Error::BadAnswer("no choices".into()))?;
        if choice.finish_reason.as_deref() == Some("length") {
            return Err(Error::BadAnswer("the answer was cut off".into()));
        }
        let content = choice.message.content.unwrap_or_default();
        let json = serde_json::from_str(&content)
            .map_err(|err| Error::BadAnswer(format!("invalid JSON: {err}")))?;
        Ok(StructuredReply {
            json,
            model: completion.model,
        })
    }
}
