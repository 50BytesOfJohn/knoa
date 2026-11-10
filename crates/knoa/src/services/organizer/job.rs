//! One job from the queue: fetch the page, ask the AI, save the answer, and decide what
//! happens to the job when something fails.

use std::{sync::Arc, time::Duration};

use gpui::{App, BackgroundExecutor};
use knoa_ai::{BookmarkInput, FilingInput, OrganizeInput, Provider};
use knoa_core::{BookmarkId, Enrichment, FolderId, Page};
use knoa_fetch::Fetcher;
use knoa_store::{Job, Store};
use tokio::runtime::Handle;

use super::{context::WorkspaceContext, now};
use crate::{
    services::{network::Network, usage},
    state::database::Database,
};

const MAX_ATTEMPTS: u32 = 5;
/// The most a provider's Retry-After can hold a job back, in case it asks for days.
const MAX_RETRY_AFTER: Duration = Duration::from_secs(3600);

/// Everything one job needs, owned so it can move to other threads.
pub(super) struct Work {
    database: Arc<Database>,
    fetcher: Arc<Fetcher>,
    provider: Option<Arc<Provider>>,
    background: BackgroundExecutor,
    network: Handle,
}

pub(super) enum Outcome {
    Done,
    Retry { error: String, after: Duration },
    Failed(String),
    NeedsAttention(String),
}

impl Work {
    pub(super) fn new(
        database: &Arc<Database>,
        fetcher: &Arc<Fetcher>,
        provider: Option<Arc<Provider>>,
        cx: &App,
    ) -> Work {
        Work {
            database: database.clone(),
            fetcher: fetcher.clone(),
            provider,
            background: cx.background_executor().clone(),
            network: Network::handle(cx),
        }
    }
}

pub(super) async fn process(job: Job, work: Arc<Work>) -> Outcome {
    let id = job.bookmark.id;
    let outcome = if job.folders_only {
        file_one(&job, &work).await
    } else {
        organize_one(&job, &work).await
    };
    // After every outcome: a job that failed may still have paid for answers.
    if let Some(provider) = &work.provider {
        usage::save(provider, work.database.clone(), &work.background).await;
    }
    let database = work.database.clone();
    let next = next_step(&outcome, job.attempts);
    let saved = work
        .background
        .spawn(async move { record(&mut database.write(), id, next) })
        .await;
    if let Err(err) = saved {
        tracing::error!(?id, "can't update the queue: {err}");
    }
    outcome
}

/// What becomes of the job's row once the job ends.
enum NextStep {
    Finished,
    RetryAfter(String, Duration),
    /// Back on the queue as it was: the organizer's pause holds it back, not the job.
    PutBack(String),
    GiveUp(String),
}

fn next_step(outcome: &Outcome, attempts: u32) -> NextStep {
    match outcome {
        Outcome::Done => NextStep::Finished,
        Outcome::Retry { error, after } if attempts + 1 < MAX_ATTEMPTS => {
            NextStep::RetryAfter(error.clone(), *after)
        }
        Outcome::NeedsAttention(error) => NextStep::PutBack(error.clone()),
        Outcome::Retry { error, .. } | Outcome::Failed(error) => NextStep::GiveUp(error.clone()),
    }
}

fn record(store: &mut Store, id: BookmarkId, next: NextStep) -> knoa_store::Result<()> {
    match next {
        NextStep::Finished => Ok(()),
        NextStep::RetryAfter(error, after) => {
            store.retry_job(id, &error, now() + after.as_secs() as i64)
        }
        NextStep::PutBack(error) => store.put_back_job(id, &error),
        NextStep::GiveUp(error) => {
            tracing::warn!(?id, "giving up on organizing: {error}");
            store.fail_job(id, &error)
        }
    }
}

async fn organize_one(job: &Job, work: &Work) -> Outcome {
    let bookmark = &job.bookmark;
    let fetcher = work.fetcher.clone();
    let url = bookmark.url.clone();
    let page = match work
        .network
        .spawn(async move { fetcher.fetch(&url).await })
        .await
    {
        Ok(Ok(page)) => page,
        Ok(Err(err)) if err.is_transient() => return retry(err.to_string(), job.attempts),
        // A page that can't be fetched still has a URL and title worth organizing.
        Ok(Err(err)) => Page {
            evidence: serde_json::json!({ "quality": "title_only", "note": err.to_string() })
                .to_string(),
            ..Page::default()
        },
        Err(err) => return Outcome::Failed(format!("fetch task failed: {err}")),
    };
    let (database, id, saved_page) = (work.database.clone(), bookmark.id, page.clone());
    let saved = work
        .background
        .spawn(async move { database.write().save_page(id, &saved_page, now()) })
        .await;
    if let Err(err) = saved {
        return Outcome::Failed(err.to_string());
    }

    let Some(provider) = work.provider.clone() else {
        let database = work.database.clone();
        let id = bookmark.id;
        return match work
            .background
            .spawn(async move { database.write().finish_page_only(id) })
            .await
        {
            Ok(()) => Outcome::Done,
            Err(err) => Outcome::Failed(err.to_string()),
        };
    };

    let database = work.database.clone();
    let workspace = bookmark.workspace;
    let context = work
        .background
        .spawn(async move { WorkspaceContext::load(&database, workspace) })
        .await;
    let context = match context {
        Ok(context) => context,
        Err(err) => return Outcome::Failed(err.to_string()),
    };
    // A link added by hand has its URL as its title until the page names it.
    let title = match &page.title {
        Some(title) if bookmark.title == bookmark.url => title.clone(),
        _ => bookmark.title.clone(),
    };
    let (url, browser_folder) = (bookmark.url.clone(), bookmark.browser_folder.clone());
    let answer = work
        .network
        .spawn(async move {
            let input = OrganizeInput {
                bookmark: BookmarkInput {
                    title: &title,
                    url: &url,
                    browser_folder: browser_folder.as_deref(),
                    evidence: Some(&page.evidence),
                    page_kind: page.kind,
                    page_fields: &page.fields,
                },
                instructions: &context.instructions,
                guideline: context.guideline.as_deref(),
                folders: &context.folders,
                tag_list: &context.tag_list,
            };
            knoa_ai::organize(&*provider, provider.classifier(), &input).await
        })
        .await;
    match answer {
        Ok(Ok(enrichment)) => save_enrichment(work, bookmark.id, enrichment).await,
        Ok(Err(err)) => ai_failure(&err, job.attempts),
        Err(err) => Outcome::Failed(format!("organize task failed: {err}")),
    }
}

/// Files an organized bookmark into folders again from its stored kind, summary and tags,
/// with one classifier request and no page fetch. A provider without a classifier picks
/// folders in the chat call, so the bookmark gets a full run instead.
async fn file_one(job: &Job, work: &Work) -> Outcome {
    let bookmark = &job.bookmark;
    let Some(provider) = work.provider.clone() else {
        // AI was turned off after queuing: nothing to ask, and the bookmark stays as it is.
        return finish_filed(work, bookmark.id, Vec::new()).await;
    };
    let (Some(kind), Some(_)) = (bookmark.kind, provider.classifier()) else {
        return organize_one(job, work).await;
    };
    let database = work.database.clone();
    let workspace = bookmark.workspace;
    let context = work
        .background
        .spawn(async move { WorkspaceContext::load(&database, workspace) })
        .await;
    let context = match context {
        Ok(context) => context,
        Err(err) => return Outcome::Failed(err.to_string()),
    };
    let bookmark = bookmark.clone();
    let answer = work
        .network
        .spawn(async move {
            let tags: Vec<String> = bookmark.tags.into_iter().map(|(tag, _)| tag).collect();
            let input = FilingInput {
                title: &bookmark.title,
                url: &bookmark.url,
                browser_folder: bookmark.browser_folder.as_deref(),
                kind,
                summary: bookmark.summary.as_deref(),
                tags: &tags,
                instructions: &context.instructions,
                folders: &context.folders,
            };
            knoa_ai::file_into_folders(&*provider, &input).await
        })
        .await;
    match answer {
        Ok(Ok(folders)) => finish_filed(work, job.bookmark.id, folders).await,
        Ok(Err(err)) => ai_failure(&err, job.attempts),
        Err(err) => Outcome::Failed(format!("folder task failed: {err}")),
    }
}

fn ai_failure(err: &knoa_ai::Error, attempts: u32) -> Outcome {
    if err.needs_user() {
        Outcome::NeedsAttention(err.to_string())
    } else if let knoa_ai::Error::RateLimited {
        retry_after: Some(wait),
    } = err
    {
        Outcome::Retry {
            error: err.to_string(),
            after: backoff(attempts).max(*wait).min(MAX_RETRY_AFTER),
        }
    } else if err.is_transient() {
        retry(err.to_string(), attempts)
    } else {
        Outcome::Failed(err.to_string())
    }
}

async fn save_enrichment(work: &Work, id: BookmarkId, enrichment: Enrichment) -> Outcome {
    let database = work.database.clone();
    match work
        .background
        .spawn(async move { database.write().finish_organized(id, &enrichment, now()) })
        .await
    {
        Ok(()) => Outcome::Done,
        Err(err) => Outcome::Failed(err.to_string()),
    }
}

async fn finish_filed(work: &Work, id: BookmarkId, folders: Vec<(FolderId, f32)>) -> Outcome {
    let database = work.database.clone();
    match work
        .background
        .spawn(async move { database.write().finish_filed(id, &folders) })
        .await
    {
        Ok(()) => Outcome::Done,
        Err(err) => Outcome::Failed(err.to_string()),
    }
}

fn retry(error: String, attempts: u32) -> Outcome {
    Outcome::Retry {
        error,
        after: backoff(attempts),
    }
}

/// Exponential backoff: 30 s, 1 min, 2 min, 4 min.
fn backoff(attempts: u32) -> Duration {
    Duration::from_secs(30 << attempts.min(6))
}
