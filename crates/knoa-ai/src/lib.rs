//! AI for Knoa, behind two small traits so providers can be swapped:
//!
//! - [`LanguageModel`]: chat with structured JSON output. Required.
//! - [`Classifier`]: yes/no and choice questions answered with probabilities. Optional;
//!   without one, the language model picks the kind and folders.
//!
//! [`Provider`] picks the implementation from settings and keeps what its requests used
//! until [`Provider::take_usage`]. [`organize`] is the provider-neutral pipeline that turns
//! a bookmark's evidence into an [`knoa_core::Enrichment`].

mod error;
mod model;
mod openrouter;
mod organize;
mod provider;
mod usage;

pub use error::Error;
pub use model::{
    Choice, Classifier, LanguageModel, Message, Question, StructuredReply, StructuredRequest,
};
pub use organize::{
    BookmarkInput, FilingInput, FolderInput, OrganizeInput, file_into_folders, organize,
};
pub use provider::{Provider, ProviderConfig, ProviderKind};
