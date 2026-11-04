//! The provider-neutral organizing pipeline: one structured chat call for kind, summary,
//! fields, extras and tags, then folders from the classifier (or from the same call when
//! the provider has none).

mod answer;
mod folders;
mod prompt;
mod schema;

use knoa_core::{Enrichment, Fields, FolderId, Kind};

use crate::{Classifier, Error, LanguageModel, StructuredRequest};

/// Enough for the answer; a model stuck in a repetition loop hits it and gets retried.
const MAX_TOKENS: u32 = 900;

pub struct BookmarkInput<'a> {
    pub title: &'a str,
    pub url: &'a str,
    pub browser_folder: Option<&'a str>,
    /// The fetch's evidence JSON, if the page was fetched.
    pub evidence: Option<&'a str>,
    /// Kind and fields read from the page itself. They win over the model's.
    pub page_kind: Option<Kind>,
    pub page_fields: &'a Fields,
}

pub struct FolderInput {
    pub id: FolderId,
    pub name: String,
    pub description: String,
}

pub struct OrganizeInput<'a> {
    pub bookmark: BookmarkInput<'a>,
    pub instructions: &'a str,
    /// Folders with auto-assign on.
    pub folders: &'a [FolderInput],
}

pub async fn organize<M: LanguageModel, C: Classifier>(
    model: &M,
    classifier: Option<&C>,
    input: &OrganizeInput<'_>,
) -> Result<Enrichment, Error> {
    let folders_in_answer = classifier.is_none();
    let request = StructuredRequest {
        messages: prompt::messages(input, folders_in_answer),
        schema_name: "bookmark",
        schema: schema::answer_schema(folders_in_answer.then_some(input.folders)),
        max_tokens: MAX_TOKENS,
    };
    let reply = model.generate(&request).await?;
    let folder_ids: Vec<FolderId> = input.folders.iter().map(|f| f.id).collect();
    let mut answer = answer::parse(reply.json, input.bookmark.page_kind, &folder_ids)?;
    for (name, value) in input.bookmark.page_fields {
        answer.fields.insert(name.clone(), value.clone());
    }

    let folders = match classifier {
        Some(classifier) => folders::pick(classifier, input, &answer).await?,
        // Chat models give no probability; record their picks as certain.
        None => answer.folders.iter().map(|id| (*id, 1.0)).collect(),
    };
    Ok(Enrichment {
        kind: answer.kind,
        summary: answer.summary,
        fields: answer.fields,
        extras: answer.extras,
        tags: answer.tags,
        folders,
        model: reply.model,
    })
}
