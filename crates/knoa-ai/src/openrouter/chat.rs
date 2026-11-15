//! Chat Completions with strict structured output.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{OpenRouter, ReportedUsage};
use crate::{Content, Error, LanguageModel, Part, StructuredReply, StructuredRequest};

#[derive(Deserialize)]
struct Completion {
    model: String,
    choices: Vec<Choice>,
    usage: Option<ReportedUsage>,
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
            .map(|m| json!({ "role": m.role, "content": content_json(&m.content) }))
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
        self.record_usage(model, completion.usage);
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

/// OpenRouter's content parts. Images go inline as data URLs, already downscaled, at
/// "high" detail: left to the model, one priced Knoa's 1024 pixel copy as a thumbnail and
/// another an original as ten times the tokens.
fn content_json(content: &Content) -> Value {
    let parts = match content {
        Content::Text(text) => return Value::from(text.as_str()),
        Content::Parts(parts) => parts,
    };
    let parts: Vec<Value> = parts
        .iter()
        .map(|part| match part {
            Part::Text(text) => json!({ "type": "text", "text": text }),
            Part::Image(image) => {
                let url = format!(
                    "data:{};base64,{}",
                    image.media_type,
                    STANDARD.encode(&image.bytes)
                );
                json!({ "type": "image_url", "image_url": { "url": url, "detail": "high" } })
            }
        })
        .collect();
    Value::from(parts)
}

#[cfg(test)]
mod tests {
    use knoa_core::AnalysisImage;

    use super::*;

    #[test]
    fn images_are_sent_as_inline_parts_after_the_text() {
        let image = AnalysisImage {
            media_type: "image/jpeg",
            bytes: b"jpeg".to_vec(),
            width: 1,
            height: 1,
        };
        let content = Content::Parts(vec![Part::Text("Look".into()), Part::Image(image)]);

        assert_eq!(
            content_json(&content),
            json!([
                { "type": "text", "text": "Look" },
                {
                    "type": "image_url",
                    "image_url": { "url": "data:image/jpeg;base64,anBlZw==", "detail": "high" },
                },
            ])
        );
        assert_eq!(content_json(&Content::Text("Hi".into())), json!("Hi"));
    }
}
