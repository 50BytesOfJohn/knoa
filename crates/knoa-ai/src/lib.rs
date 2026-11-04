//! AI for Knoa, behind two small traits so providers can be swapped:
//!
//! - [`LanguageModel`]: chat with structured JSON output. Required.
//! - [`Classifier`]: yes/no questions answered with probabilities. Optional; without one,
//!   folders are picked by the language model in the same call.
//!
//! [`Provider`] picks the implementation from settings. [`organize`] is the provider-neutral
//! pipeline that turns a bookmark's evidence into an [`knoa_core::Enrichment`].

mod error;
mod model;
mod openrouter;
mod organize;
mod provider;

pub use error::Error;
pub use model::{Classifier, LanguageModel, Message, Question, StructuredReply, StructuredRequest};
pub use organize::{BookmarkInput, FolderInput, OrganizeInput, organize};
pub use provider::{Provider, ProviderConfig, ProviderKind};
