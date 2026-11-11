//! The capabilities Knoa needs from an AI provider.

use serde_json::Value;

use crate::Error;

/// `Send` on the desktop, where requests run on a multi-threaded runtime. Nothing in a
/// Worker is `Send` (JavaScript promises stay on their thread), and nothing needs to be.
#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSend: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + ?Sized> MaybeSend for T {}
#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> MaybeSend for T {}

/// `Sync` on the desktop; see [`MaybeSend`].
#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSync: Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Sync + ?Sized> MaybeSync for T {}
#[cfg(target_arch = "wasm32")]
pub trait MaybeSync {}
#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> MaybeSync for T {}

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

pub trait LanguageModel: MaybeSend + MaybeSync {
    /// Generates JSON matching `request.schema`. A provider may retry with a fallback model
    /// when the first answer is malformed or truncated.
    fn generate(
        &self,
        request: &StructuredRequest,
    ) -> impl Future<Output = Result<StructuredReply, Error>> + MaybeSend;
}

/// A yes/no question about some state, answered with a probability of "yes".
#[derive(Debug, Clone)]
pub struct Question {
    pub key: String,
    pub instructions: String,
    pub yes: String,
    pub no: String,
}

/// A question about some state with exactly one answer from a fixed set of options.
#[derive(Debug, Clone)]
pub struct Choice {
    pub instructions: String,
    /// Each option's key and when it is the answer.
    pub options: Vec<(String, String)>,
}

pub trait Classifier: MaybeSend + MaybeSync {
    /// The probability of "yes" for each question, in order.
    fn classify(
        &self,
        state: &Value,
        questions: &[Question],
    ) -> impl Future<Output = Result<Vec<f32>, Error>> + MaybeSend;

    /// The probability of each option, in order.
    fn choose(
        &self,
        state: &Value,
        choice: &Choice,
    ) -> impl Future<Output = Result<Vec<f32>, Error>> + MaybeSend;
}
