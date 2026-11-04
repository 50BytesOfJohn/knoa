//! Works through the organizing queue: fetch each page, ask the AI, store the result.
//!
//! Up to `CONCURRENCY` bookmarks are in flight. Network work runs on tokio, database work
//! on the background executor; this loop only coordinates. Without an AI provider, pages
//! are still fetched so cards get their image and description.

use std::{sync::Arc, time::Duration};

use futures::{FutureExt as _, StreamExt as _, channel::mpsc, stream::FuturesUnordered};
use gpui::{App, AppContext as _, AsyncApp, BackgroundExecutor, Context, Entity, Global, Task};
use knoa_ai::{BookmarkInput, FolderInput, OrganizeInput, Provider};
use knoa_core::{BookmarkId, Enrichment, Page, WorkspaceId};
use knoa_fetch::Fetcher;
use knoa_store::Job;
use tokio::runtime::Handle;

use crate::{
    services::network::Network,
    state::library::{Library, SharedStore, lock},
};

/// About four network requests at a time, as the architecture plans.
const CONCURRENCY: usize = 4;
const MAX_ATTEMPTS: u32 = 5;

/// What the AI side of the app is doing, for the sidebar indicator and settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiStatus {
    Off,
    Ready,
    /// The provider refuses requests until the user acts (bad key, no credits).
    NeedsAttention(String),
}

pub struct Organizer {
    provider: Option<Arc<Provider>>,
    status: AiStatus,
    wake: mpsc::UnboundedSender<()>,
    _run: Task<()>,
}

struct OrganizerGlobal(Entity<Organizer>);

impl Global for OrganizerGlobal {}

/// Everything one job needs, owned so it can move to other threads.
struct Work {
    store: SharedStore,
    fetcher: Arc<Fetcher>,
    provider: Option<Arc<Provider>>,
    background: BackgroundExecutor,
    network: Handle,
}

enum Outcome {
    Done,
    Retry { error: String, after: Duration },
    Failed(String),
    NeedsAttention(String),
}

impl Organizer {
    pub fn init(library: Entity<Library>, fetcher: Fetcher, cx: &mut App) {
        let fetcher = Arc::new(fetcher);
        let organizer = cx.new(|cx| {
            let (wake, wake_rx) = mpsc::unbounded();
            let run =
                cx.spawn(async move |this, cx| run(this, library, fetcher, wake_rx, cx).await);
            Organizer {
                provider: None,
                status: AiStatus::Off,
                wake,
                _run: run,
            }
        });
        cx.set_global(OrganizerGlobal(organizer));
    }

    pub fn entity(cx: &App) -> Entity<Organizer> {
        cx.global::<OrganizerGlobal>().0.clone()
    }

    /// New work may be waiting.
    pub fn wake(cx: &mut App) {
        if let Some(global) = cx.try_global::<OrganizerGlobal>() {
            let _ = global.0.read(cx).wake.unbounded_send(());
        }
    }

    pub fn status(&self) -> &AiStatus {
        &self.status
    }

    pub fn set_provider(&mut self, provider: Option<Provider>, cx: &mut Context<'_, Organizer>) {
        self.status = if provider.is_some() {
            AiStatus::Ready
        } else {
            AiStatus::Off
        };
        self.provider = provider.map(Arc::new);
        let _ = self.wake.unbounded_send(());
        cx.notify();
    }
}

async fn run(
    this: gpui::WeakEntity<Organizer>,
    library: Entity<Library>,
    fetcher: Arc<Fetcher>,
    mut wake: mpsc::UnboundedReceiver<()>,
    cx: &mut AsyncApp,
) {
    let store = cx.update(|cx| library.read(cx).store());
    let requeued = cx.background_spawn({
        let store = store.clone();
        async move { lock(&store).requeue_interrupted() }
    });
    if let Err(err) = requeued.await {
        tracing::error!("can't requeue interrupted jobs: {err}");
    }
    let mut running = FuturesUnordered::new();
    loop {
        let free = CONCURRENCY - running.len();
        if free > 0 {
            let Ok(work) = this.update(cx, |this, cx| {
                Arc::new(work_for(this, &store, &fetcher, cx))
            }) else {
                return;
            };
            let jobs = cx.background_spawn({
                let store = store.clone();
                async move { lock(&store).take_jobs(now(), free) }
            });
            match jobs.await {
                Ok(jobs) => running.extend(
                    jobs.into_iter()
                        .map(|job| process(job, work.clone()).boxed()),
                ),
                Err(err) => tracing::error!("can't read the organizing queue: {err}"),
            }
            if !running.is_empty() {
                let () = library.update(cx, Library::refresh_soon);
            }
        }

        if running.is_empty() {
            let wait = next_wait(&store, cx).await;
            futures::select! {
                _ = wake.next() => {}
                () = cx.background_executor().timer(wait).fuse() => {}
            }
            continue;
        }

        futures::select! {
            outcome = running.select_next_some() => {
                if let Outcome::NeedsAttention(message) = outcome {
                    let _ = this.update(cx, |this, cx| {
                        this.status = AiStatus::NeedsAttention(message);
                        cx.notify();
                    });
                }
                let () = library.update(cx, Library::refresh_soon);
            }
            _ = wake.next() => {}
        }
    }
}

fn work_for(organizer: &Organizer, store: &SharedStore, fetcher: &Arc<Fetcher>, cx: &App) -> Work {
    Work {
        store: store.clone(),
        fetcher: fetcher.clone(),
        provider: organizer.provider.clone(),
        background: cx.background_executor().clone(),
        network: Network::handle(cx),
    }
}

/// How long to sleep when nothing is running: until the next retry, or an hour.
async fn next_wait(store: &SharedStore, cx: &AsyncApp) -> Duration {
    let store = store.clone();
    let next = cx
        .background_spawn(async move { lock(&store).next_job_at() })
        .await
        .ok()
        .flatten();
    match next {
        Some(at) => Duration::from_secs((at - now()).clamp(1, 3600) as u64),
        None => Duration::from_secs(3600),
    }
}

async fn process(job: Job, work: Arc<Work>) -> Outcome {
    let id = job.bookmark.id;
    let outcome = organize_one(&job, &work).await;
    let store = work.store.clone();
    let saved = work
        .background
        .spawn({
            let attempts = job.attempts;
            let outcome = match &outcome {
                Outcome::Done => None,
                Outcome::Retry { error, after } if attempts + 1 < MAX_ATTEMPTS => {
                    Some((error.clone(), Some(*after)))
                }
                Outcome::NeedsAttention(error) => {
                    Some((error.clone(), Some(Duration::from_secs(300))))
                }
                Outcome::Retry { error, .. } | Outcome::Failed(error) => {
                    Some((error.clone(), None))
                }
            };
            async move { record_failure(&store, id, outcome) }
        })
        .await;
    if let Err(err) = saved {
        tracing::error!(?id, "can't update the queue: {err}");
    }
    outcome
}

fn record_failure(
    store: &SharedStore,
    id: BookmarkId,
    failure: Option<(String, Option<Duration>)>,
) -> knoa_store::Result<()> {
    let mut store = lock(store);
    match failure {
        None => Ok(()),
        Some((error, Some(after))) => store.retry_job(id, &error, now() + after.as_secs() as i64),
        Some((error, None)) => {
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
    let (store, id, saved_page) = (work.store.clone(), bookmark.id, page.clone());
    let saved = work
        .background
        .spawn(async move { save_page(&store, id, &saved_page) })
        .await;
    if let Err(err) = saved {
        return Outcome::Failed(err.to_string());
    }

    let Some(provider) = work.provider.clone() else {
        let store = work.store.clone();
        let id = bookmark.id;
        return match work
            .background
            .spawn(async move { lock(&store).finish_page_only(id) })
            .await
        {
            Ok(()) => Outcome::Done,
            Err(err) => Outcome::Failed(err.to_string()),
        };
    };

    let store = work.store.clone();
    let workspace = bookmark.workspace;
    let context = work
        .background
        .spawn(async move { WorkspaceContext::load(&store, workspace) })
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
            };
            knoa_ai::organize(&*provider, provider.classifier(), &input).await
        })
        .await;
    match answer {
        Ok(Ok(enrichment)) => save_enrichment(work, bookmark.id, enrichment).await,
        Ok(Err(err)) if err.needs_user() => Outcome::NeedsAttention(err.to_string()),
        Ok(Err(err)) if err.is_transient() => retry(err.to_string(), job.attempts),
        Ok(Err(err)) => Outcome::Failed(err.to_string()),
        Err(err) => Outcome::Failed(format!("organize task failed: {err}")),
    }
}

/// What the bookmark's workspace tells the AI: its instructions, guideline and the folders
/// it may file into.
struct WorkspaceContext {
    instructions: String,
    guideline: Option<String>,
    folders: Vec<FolderInput>,
}

impl WorkspaceContext {
    fn load(store: &SharedStore, id: WorkspaceId) -> knoa_store::Result<WorkspaceContext> {
        let store = lock(store);
        let workspace = store.workspace(id)?;
        let folders = store
            .folders(id)?
            .into_iter()
            .filter(|f| f.auto_assign)
            .map(|f| FolderInput {
                id: f.id,
                name: f.name,
                description: f.description,
            })
            .collect();
        Ok(WorkspaceContext {
            instructions: workspace
                .as_ref()
                .map(|w| w.instructions.clone())
                .unwrap_or_default(),
            guideline: workspace
                .as_ref()
                .and_then(|w| w.screening.active_guideline())
                .map(str::to_owned),
            folders,
        })
    }
}

fn save_page(store: &SharedStore, id: BookmarkId, page: &Page) -> knoa_store::Result<()> {
    lock(store).save_page(id, page, now())
}

async fn save_enrichment(work: &Work, id: BookmarkId, enrichment: Enrichment) -> Outcome {
    let store = work.store.clone();
    match work
        .background
        .spawn(async move { lock(&store).finish_organized(id, &enrichment, now()) })
        .await
    {
        Ok(()) => Outcome::Done,
        Err(err) => Outcome::Failed(err.to_string()),
    }
}

/// Exponential backoff: 30 s, 1 min, 2 min, 4 min.
fn retry(error: String, attempts: u32) -> Outcome {
    Outcome::Retry {
        error,
        after: Duration::from_secs(30 << attempts.min(6)),
    }
}

pub fn now() -> i64 {
    jiff::Timestamp::now().as_second()
}
