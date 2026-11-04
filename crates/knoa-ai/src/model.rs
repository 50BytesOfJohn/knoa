//! The capabilities Knoa needs from an AI provider.

use serde_json::Value;

use crate::Error;

#[derive(Debug, Clone)]
pub struct Message {
    pub role: &'static str,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Message {
        Message {
            role: "system",
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Message {
        Message {
            role: "user",
            content: content.into(),
        }
    }
}

/// A chat request whose answer must match a JSON schema.
#[derive(Debug, Clone)]
pub struct StructuredRequest {
    /// Keep the leading messages identical across requests: providers cache that prefix.
    pub messages: Vec<Message>,
    pub schema_name: &'static str,
    pub schema: Value,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct StructuredReply {
    pub json: Value,
    /// The model that actually answered.
    pub model: String,
}

pub trait LanguageModel: Send + Sync {
    /// Generates JSON matching `request.schema`. A provider may retry with a fallback model
    /// when the first answer is malformed or truncated.
    fn generate(
        &self,
        request: &StructuredRequest,
    ) -> impl Future<Output = Result<StructuredReply, Error>> + Send;
}

/// A yes/no question about some state, answered with a probability of "yes".
#[derive(Debug, Clone)]
pub struct Question {
    pub key: String,
    pub instructions: String,
    pub yes: String,
    pub no: String,
}

pub trait Classifier: Send + Sync {
    /// The probability of "yes" for each question, in order.
    fn classify(
        &self,
        state: &Value,
        questions: &[Question],
    ) -> impl Future<Output = Result<Vec<f32>, Error>> + Send;
}
