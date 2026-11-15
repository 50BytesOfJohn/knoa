//! The provider-neutral organizing pipeline: the kind first, then one structured chat call
//! for that kind's fields, summary, extras, tags and, when the workspace has a guideline,
//! whether the bookmark fits; then folders. The classifier picks the kind and folders; a
//! provider without one asks the chat model.

mod answer;
mod folders;
mod kind;
mod prompt;
mod schema;

pub use folders::{FilingInput, file_into_folders};
use knoa_core::{
    AnalysisImage, Enrichment, Fields, FolderId, Kind, TagList, link_file_name, site_of,
};

use crate::{Classifier, Error, LanguageModel, StructuredRequest};

/// Enough for the answer; a model stuck in a repetition loop hits it and gets retried.
const MAX_TOKENS: u32 = 900;

pub struct BookmarkInput<'a> {
    pub title: &'a str,
    pub url: &'a str,
    pub browser_folder: Option<&'a str>,
    /// The fetch's evidence JSON, if the page was fetched.
    pub evidence: Option<&'a str>,
    /// Kind and fields read from the page itself. They win over the models'.
    pub page_kind: Option<Kind>,
    pub page_fields: &'a Fields,
    /// For a link straight to an image file: a small copy for the model to look at.
    pub image: Option<&'a AnalysisImage>,
}

pub struct FolderInput {
    pub id: FolderId,
    pub name: String,
    pub description: String,
}

pub struct OrganizeInput<'a> {
    pub bookmark: BookmarkInput<'a>,
    /// The workspace's organizing instructions.
    pub instructions: &'a str,
    /// What the workspace keeps, when it screens bookmarks. The bookmark is still
    /// organized either way, so keeping it later costs nothing.
    pub guideline: Option<&'a str>,
    /// Folders with auto-assign on.
    pub folders: &'a [FolderInput],
    /// The tags the user wants used here, if any.
    pub tag_list: &'a TagList,
}

pub async fn organize<M: LanguageModel, C: Classifier>(
    model: &M,
    classifier: Option<&C>,
    input: &OrganizeInput<'_>,
) -> Result<Enrichment, Error> {
    let kind = kind::pick(model, classifier, input).await?;
    let folders_in_answer = classifier.is_none();
    let asks = schema::Asks {
        folders: folders_in_answer.then_some(input.folders),
        screening: input.guideline.is_some(),
        title: needs_title(&input.bookmark),
    };
    let request = |image| StructuredRequest {
        messages: prompt::messages(input, kind, folders_in_answer, image),
        schema_name: "bookmark",
        schema: schema::answer_schema(kind, &asks),
        max_tokens: MAX_TOKENS,
    };
    let reply = match model.generate(&request(input.bookmark.image)).await {
        // A user's own models may not take images; the details still say a lot. Only when
        // every model refused it unbilled: an unusable answer was paid for, and fails as
        // any other for the job's own retries.
        Err(err) if err.is_refusal() && input.bookmark.image.is_some() => {
            tracing::debug!("no model took the image, organizing without it: {err}");
            model.generate(&request(None)).await?
        }
        reply => reply?,
    };
    let folder_ids: Vec<FolderId> = input.folders.iter().map(|f| f.id).collect();
    let mut answer = answer::parse(reply.json, kind, &folder_ids, input.tag_list)?;
    for (name, value) in input.bookmark.page_fields {
        answer.fields.insert(name.clone(), value.clone());
    }

    let folders = match classifier {
        // A bookmark set aside isn't filed; it gets folders once the user keeps it.
        _ if answer.set_aside.is_some() => Vec::new(),
        Some(classifier) => file_into_folders(classifier, &filing_input(input, &answer)).await?,
        // Chat models give no probability; record their picks as certain.
        None => answer.folders.iter().map(|id| (*id, 1.0)).collect(),
    };
    let title = asks
        .title
        .then(|| usable_title(answer.title.take(), input.bookmark.url))
        .flatten();
    Ok(Enrichment {
        kind: answer.kind,
        title,
        summary: answer.summary,
        fields: answer.fields,
        extras: answer.extras,
        tags: answer.tags,
        folders,
        set_aside: answer.set_aside,
        model: reply.model,
    })
}

/// A link added by hand is titled with its URL until something names it. Pages usually do,
/// an image file never does.
fn needs_title(bookmark: &BookmarkInput<'_>) -> bool {
    let title = bookmark.title.trim();
    title.is_empty() || title == bookmark.url
}

/// The model's title, unless it left it empty or repeated the URL, which some endpoints do
/// whatever the schema says. Asking again would double the cost, so the link's file name
/// or site stands in: still better than the URL.
fn usable_title(title: Option<String>, url: &str) -> Option<String> {
    title.filter(|title| title != url).or_else(|| {
        tracing::debug!("the answer has no usable title, naming the bookmark after its link");
        link_file_name(url).or_else(|| Some(site_of(url)).filter(|site| !site.is_empty()))
    })
}

fn filing_input<'a>(input: &'a OrganizeInput<'_>, answer: &'a answer::Answer) -> FilingInput<'a> {
    FilingInput {
        title: input.bookmark.title,
        url: input.bookmark.url,
        browser_folder: input.bookmark.browser_folder,
        kind: answer.kind,
        summary: answer.summary.as_deref(),
        tags: &answer.tags,
        instructions: input.instructions,
        folders: input.folders,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use serde_json::json;

    use super::*;
    use crate::{Content, Part, Provider, StructuredReply, error::Failures};

    /// How a fake chat model treats a request.
    #[derive(Clone, Copy)]
    enum Model {
        /// Refuses images, unbilled, and answers text.
        Blind,
        /// Takes the image and answers with something unusable, which was billed.
        Garbled,
    }

    /// Chat models in fallback order, recording whether each request carried the image.
    struct Models {
        models: Vec<Model>,
        requests: Mutex<Vec<bool>>,
    }

    impl Models {
        fn new(models: &[Model]) -> Models {
            Models {
                models: models.to_vec(),
                requests: Mutex::default(),
            }
        }

        fn requests(&self) -> Vec<bool> {
            self.requests.lock().expect("lock poisoned").clone()
        }
    }

    impl LanguageModel for Models {
        #[expect(
            clippy::unused_async_trait_impl,
            reason = "a real provider awaits its requests"
        )]
        async fn generate(&self, request: &StructuredRequest) -> Result<StructuredReply, Error> {
            let with_image = request.messages.iter().any(|message| {
                matches!(&message.content, Content::Parts(parts)
                    if parts.iter().any(|part| matches!(part, Part::Image(_))))
            });
            let mut failures = Failures::default();
            for model in &self.models {
                self.requests
                    .lock()
                    .expect("lock poisoned")
                    .push(with_image);
                let reply = match (model, with_image) {
                    (Model::Blind, true) => Err(Error::Provider {
                        status: 404,
                        message: "no endpoints support image input".into(),
                    }),
                    (Model::Garbled, true) => Err(Error::BadAnswer("invalid JSON".into())),
                    (_, false) => Ok(StructuredReply {
                        json: json!({ "summary": "A picture.", "fields": {}, "extras": [], "tags": [] }),
                        model: "fake".into(),
                    }),
                };
                match reply {
                    Ok(reply) => return Ok(reply),
                    Err(err) => failures.add(err),
                }
            }
            Err(failures.into_error())
        }
    }

    async fn organize_picture(models: &Models) -> Result<Enrichment, Error> {
        let image = AnalysisImage {
            media_type: "image/jpeg",
            bytes: vec![0xff, 0xd8],
            width: 1024,
            height: 576,
        };
        let input = OrganizeInput {
            bookmark: BookmarkInput {
                title: "A picture",
                url: "https://a.dev/picture.jpg",
                browser_folder: None,
                evidence: None,
                page_kind: Some(Kind::Image),
                page_fields: &Fields::new(),
                image: Some(&image),
            },
            instructions: "",
            guideline: None,
            folders: &[],
            tag_list: &TagList::default(),
        };
        organize(models, None::<&Provider>, &input).await
    }

    /// Answers every request with the same JSON.
    struct Answers(serde_json::Value);

    impl LanguageModel for Answers {
        #[expect(
            clippy::unused_async_trait_impl,
            reason = "a real provider awaits its requests"
        )]
        async fn generate(&self, _: &StructuredRequest) -> Result<StructuredReply, Error> {
            Ok(StructuredReply {
                json: self.0.clone(),
                model: "fake".into(),
            })
        }
    }

    #[tokio::test]
    async fn a_link_the_model_leaves_untitled_is_named_after_its_file() {
        let url = "https://w.wallhaven.cc/full/6l/wallhaven-%C3%A9.jpg";
        let model = Answers(json!({
            "title": " ", "summary": "A café.", "fields": {}, "extras": [], "tags": [],
        }));
        let input = OrganizeInput {
            bookmark: BookmarkInput {
                title: url,
                url,
                browser_folder: None,
                evidence: None,
                page_kind: Some(Kind::Image),
                page_fields: &Fields::new(),
                image: None,
            },
            instructions: "",
            guideline: None,
            folders: &[],
            tag_list: &TagList::default(),
        };

        let enrichment = organize(&model, None::<&Provider>, &input)
            .await
            .expect("organized");

        assert_eq!(enrichment.title.as_deref(), Some("wallhaven-é.jpg"));
    }

    #[tokio::test]
    async fn a_picture_every_model_refuses_is_organized_once_more_without_it() {
        let models = Models::new(&[Model::Blind, Model::Blind]);

        let enrichment = organize_picture(&models).await.expect("organized");

        assert_eq!(enrichment.summary.as_deref(), Some("A picture."));
        assert_eq!(models.requests(), [true, true, false]);
    }

    #[tokio::test]
    async fn a_bad_answer_to_the_picture_is_not_asked_again_without_it() {
        let models = Models::new(&[Model::Garbled]);

        let result = organize_picture(&models).await;

        assert!(matches!(result, Err(Error::BadAnswer(_))));
        assert_eq!(models.requests(), [true]);
    }

    #[tokio::test]
    async fn one_bad_answer_among_refusals_is_not_asked_again_without_the_picture() {
        let models = Models::new(&[Model::Garbled, Model::Blind]);

        let result = organize_picture(&models).await;

        assert!(matches!(result, Err(Error::BadAnswer(_))));
        assert_eq!(models.requests(), [true, true]);
    }
}
