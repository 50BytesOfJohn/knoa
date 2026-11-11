//! One job, the desktop's way: fetch the page, pick the kind, organize, file into folders.
//! A folders-only job skips straight to filing.

use std::time::Duration;

use knoa_ai::{
    BookmarkInput, FilingInput, FolderInput, OrganizeInput, Provider, ProviderConfig, ProviderKind,
};
use knoa_core::{Enrichment, FolderId, Kind, ListedTag, Page, TagList, normalize_tag, site_of};
use knoa_fetch::{Fetched, Fetcher};

use crate::{
    job::{self, Job, JobResult},
    usage,
};

/// The queue's own delay for network trouble and provider hiccups.
const RETRY_AFTER: Duration = Duration::from_secs(60);
/// Our key was rejected or ran dry. The desktop probes every 5 minutes in that state.
const NEEDS_US_RETRY: Duration = Duration::from_secs(300);
/// The most a provider's Retry-After can hold a job back.
const MAX_RETRY_AFTER: Duration = Duration::from_secs(3600);

pub enum Outcome {
    /// Done, or failed for good with `error` set: either way the result is delivered.
    Finished(Box<JobResult>),
    /// Worth trying again later.
    Retry { error: String, after: Duration },
}

pub async fn run(job: &Job, api_key: String) -> Outcome {
    let provider = match Provider::new(
        &ProviderConfig::default_for(ProviderKind::OpenRouter),
        api_key,
    ) {
        Ok(provider) => provider,
        Err(err) => return failed(job, err.to_string(), Vec::new()),
    };
    let folders = folder_inputs(job);
    let outcome = match existing_kind(job) {
        Some(kind) if provider.classifier().is_some() => file(job, &provider, kind, &folders).await,
        _ => organize(job, &provider, &folders).await,
    };
    let usage = usage::by_model(provider.kind().id(), provider.take_usage());
    match outcome {
        Ok(mut result) => {
            result.usage = usage;
            Outcome::Finished(result)
        }
        Err(Failure::Retry { error, after }) => Outcome::Retry { error, after },
        Err(Failure::Permanent(error)) => failed(job, error, usage),
    }
}

enum Failure {
    Retry { error: String, after: Duration },
    Permanent(String),
}

fn existing_kind(job: &Job) -> Option<Kind> {
    let existing = job.existing.as_ref().filter(|_| job.folders_only)?;
    Kind::parse(&existing.kind)
}

/// Files an organized bookmark again from its stored kind, summary and tags.
async fn file(
    job: &Job,
    provider: &Provider,
    kind: Kind,
    folders: &[FolderInput],
) -> Result<Box<JobResult>, Failure> {
    let existing = job.existing.as_ref();
    let input = FilingInput {
        title: &job.title,
        url: &job.url,
        browser_folder: job.browser_folder.as_deref(),
        kind,
        summary: existing.and_then(|e| e.summary.as_deref()),
        tags: existing.map(|e| e.tags.as_slice()).unwrap_or_default(),
        instructions: &job.workspace.instructions,
        folders,
    };
    let picked = knoa_ai::file_into_folders(provider, &input)
        .await
        .map_err(|err| ai_failure(&err))?;
    Ok(Box::new(JobResult {
        folders: folder_picks(job, &picked),
        ..empty_result(job)
    }))
}

async fn organize(
    job: &Job,
    provider: &Provider,
    folders: &[FolderInput],
) -> Result<Box<JobResult>, Failure> {
    let Fetched { page, preview } = fetch(&job.url).await?;
    // A link added by hand has its URL as its title until the page names it.
    let new_title = page.title.clone().filter(|_| job.title == job.url);
    let title = new_title.as_deref().unwrap_or(&job.title);
    let tag_list = tag_list(&job.workspace);
    let input = OrganizeInput {
        bookmark: BookmarkInput {
            title,
            url: &job.url,
            browser_folder: job.browser_folder.as_deref(),
            evidence: Some(&page.evidence),
            page_kind: page.kind,
            page_fields: &page.fields,
        },
        instructions: &job.workspace.instructions,
        guideline: job.workspace.guideline.as_deref(),
        folders,
        tag_list: &tag_list,
    };
    let enrichment = knoa_ai::organize(provider, provider.classifier(), &input)
        .await
        .map_err(|err| ai_failure(&err))?;
    let mut result = organized(job, enrichment);
    result.title = new_title;
    result.description = page.description;
    result.page_text = Some(page.text).filter(|text| !text.is_empty());
    result.page_evidence = Some(page.evidence);
    result.image = preview.map(|p| job::Image {
        url: p.url.into(),
        width: p.width,
        height: p.height,
    });
    Ok(result)
}

async fn fetch(url: &str) -> Result<Fetched, Failure> {
    let client = Fetcher::new().map_err(|err| Failure::Permanent(err.to_string()))?;
    match client.fetch_with_preview(url).await {
        Ok(fetched) => Ok(fetched),
        Err(err) if err.is_transient() => Err(Failure::Retry {
            error: err.to_string(),
            after: RETRY_AFTER,
        }),
        // A page that can't be fetched still has a URL and title worth organizing.
        Err(err) => Ok(Page {
            evidence: serde_json::json!({ "quality": "title_only", "note": err.to_string() })
                .to_string(),
            ..Page::default()
        }
        .into()),
    }
}

fn organized(job: &Job, enrichment: Enrichment) -> Box<JobResult> {
    Box::new(JobResult {
        kind: Some(enrichment.kind.slug().to_owned()),
        summary: enrichment.summary,
        fields: enrichment.fields,
        extras: enrichment
            .extras
            .into_iter()
            .map(|(label, value)| job::Extra { label, value })
            .collect(),
        tags: enrichment.tags,
        folders: folder_picks(job, &enrichment.folders),
        set_aside: enrichment.set_aside,
        model: Some(enrichment.model),
        ..empty_result(job)
    })
}

fn failed(job: &Job, error: String, usage: Vec<job::ModelUsage>) -> Outcome {
    let mut result = failure(job, error);
    result.usage = usage;
    Outcome::Finished(result)
}

/// The result for a job that failed for good: nothing changes but the error.
pub fn failure(job: &Job, error: String) -> Box<JobResult> {
    Box::new(JobResult {
        error: Some(error),
        ..empty_result(job)
    })
}

fn empty_result(job: &Job) -> JobResult {
    JobResult {
        bookmark_id: job.bookmark_id.clone(),
        site: site_of(&job.url),
        ..JobResult::default()
    }
}

fn ai_failure(err: &knoa_ai::Error) -> Failure {
    let after = match err {
        _ if err.needs_user() => NEEDS_US_RETRY,
        knoa_ai::Error::RateLimited {
            retry_after: Some(wait),
        } => (*wait).clamp(RETRY_AFTER, MAX_RETRY_AFTER),
        _ if err.is_transient() => RETRY_AFTER,
        _ => return Failure::Permanent(err.to_string()),
    };
    Failure::Retry {
        error: err.to_string(),
        after,
    }
}

/// The AI sees folders by position; the job's ids are the API's and can be anything.
fn folder_inputs(job: &Job) -> Vec<FolderInput> {
    job.folders
        .iter()
        .enumerate()
        .map(|(index, folder)| FolderInput {
            id: FolderId(index as i64),
            name: folder.name.clone(),
            description: folder.description.clone(),
        })
        .collect()
}

fn folder_picks(job: &Job, picked: &[(FolderId, f32)]) -> Vec<job::FolderPick> {
    picked
        .iter()
        .filter_map(|&(FolderId(index), probability)| {
            let folder = job.folders.get(usize::try_from(index).ok()?)?;
            Some(job::FolderPick {
                id: folder.id.clone(),
                probability,
            })
        })
        .collect()
}

/// Spelled and sorted the way the desktop stores a tag list.
fn tag_list(workspace: &job::Workspace) -> TagList {
    let mut tags: Vec<ListedTag> = workspace
        .tag_list
        .iter()
        .map(|t| ListedTag {
            name: normalize_tag(&t.tag),
            description: t.description.trim().to_owned(),
        })
        .filter(|t| !t.name.is_empty())
        .collect();
    tags.sort_by(|a, b| a.name.cmp(&b.name));
    tags.dedup_by(|a, b| a.name == b.name);
    TagList {
        tags,
        only_these: workspace.only_listed_tags,
    }
}
