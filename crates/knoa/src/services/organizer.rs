//! Works through the organizing queue: fetch each page, ask the AI, store the result.
//!
//! Up to `CONCURRENCY` bookmarks are in flight. Network work runs on tokio, database work
//! on the background executor; this loop only coordinates. Without an AI provider, pages
//! are still fetched so cards get their image and description. While the provider rejects
//! the key or has no credits, the queue pauses and tries one job every `PROBE_EVERY`.

mod context;
mod job;
mod work;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{FutureExt as _, StreamExt as _, channel::mpsc, stream::FuturesUnordered};
use gpui::{App, AppContext as _, AsyncApp, Context, Entity, Global, Task};
use knoa_ai::Provider;
use knoa_fetch::Fetcher;
use knoa_store::Job;

use self::{
    job::{Outcome, process},
    work::Work,
};
use crate::{
    services::generation,
    state::{database::Database, library::Library},
};

/// About four network requests at a time, as the architecture plans.
const CONCURRENCY: usize = 4;
/// How often a paused queue tries one job to see whether the provider works again.
const PROBE_EVERY: Duration = Duration::from_secs(300);

/// What the AI side of the app is doing, for the sidebar indicator and settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiStatus {
    Off,
    Ready,
    /// The provider refuses requests until the user acts (bad key, no credits).
    NeedsAttention(String),
    /// The account's cloud organizer does the work; this Mac's queue waits.
    Cloud,
}

pub struct Organizer {
    provider: Option<Arc<Provider>>,
    status: AiStatus,
    /// Set while the provider needs the user: the queue is paused, and at this time one
    /// job goes out as a probe to see whether it works again.
    probe_at: Option<Instant>,
    /// The linked account's cloud organizer runs the queue instead (Pro). The provider and
    /// its key stay, for when that ends.
    cloud: bool,
    /// The reason last given for `cloud`, so a new reason is logged once.
    cloud_why: Option<&'static str>,
    generation: generation::Generation,
    wake: mpsc::UnboundedSender<()>,
    _run: Task<()>,
}

struct OrganizerGlobal(Entity<Organizer>);

impl Global for OrganizerGlobal {}

impl Organizer {
    pub fn init(library: Entity<Library>, fetcher: Arc<Fetcher>, cx: &mut App) {
        let organizer = cx.new(|cx| {
            let (wake, wake_rx) = mpsc::unbounded();
            let run =
                cx.spawn(async move |this, cx| run(this, library, fetcher, wake_rx, cx).await);
            Organizer {
                provider: None,
                status: AiStatus::Off,
                probe_at: None,
                cloud: false,
                cloud_why: None,
                generation: generation::Generation::default(),
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
        self.provider = provider.map(Arc::new);
        self.status = self.connected_status();
        self.probe_at = None;
        let _ = self.wake.unbounded_send(());
        cx.notify();
    }

    /// Hands the queue to the cloud organizer, or takes it back. `why` goes to the log
    /// whenever it changes, even when the state doesn't, so "why is nothing organizing"
    /// can be answered from it.
    pub fn set_cloud(&mut self, cloud: bool, why: &'static str, cx: &mut Context<'_, Organizer>) {
        let changed = self.cloud != cloud;
        if changed || self.cloud_why != Some(why) {
            match (cloud, changed) {
                (true, _) => tracing::info!("local organizing paused: {why}"),
                (false, true) => tracing::info!("local organizing resumed: {why}"),
                (false, false) => tracing::info!("local organizing allowed: {why}"),
            }
            self.cloud_why = Some(why);
        }
        if changed {
            self.generation.advance();
            self.cloud = cloud;
            self.status = self.connected_status();
            self.probe_at = None;
            let _ = self.wake.unbounded_send(());
            cx.notify();
        }
    }

    fn connected_status(&self) -> AiStatus {
        if self.cloud {
            AiStatus::Cloud
        } else if self.provider.is_some() {
            AiStatus::Ready
        } else {
            AiStatus::Off
        }
    }

    /// Decides how many jobs to start now. Taking a probe moves the next probe on, so
    /// one comes at most every `PROBE_EVERY`, even when the queue turns out empty.
    fn jobs_to_take_now(&mut self, in_flight: usize) -> usize {
        if self.cloud {
            return 0;
        }
        let now = Instant::now();
        let take = jobs_to_take(CONCURRENCY - in_flight, in_flight, self.probe_at, now);
        if take > 0 && self.probe_at.is_some() {
            self.probe_at = Some(now + PROBE_EVERY);
        }
        take
    }

    /// Pauses the queue when the provider needs the user, and resumes it when a job
    /// gets through again (topped-up credits need no new key).
    fn job_ended(&mut self, outcome: &Outcome, cx: &mut Context<'_, Organizer>) {
        match outcome {
            // With AI turned off since the job started, the next run is page-only and needs
            // nothing from the user.
            Outcome::NeedsAttention(message) if self.provider.is_some() && !self.cloud => {
                self.status = AiStatus::NeedsAttention(message.clone());
                self.probe_at = Some(Instant::now() + PROBE_EVERY);
                cx.notify();
            }
            // A folders-only job with no folders to ask about also ends here without
            // asking the provider; the next real job pauses the queue again.
            Outcome::Done if self.probe_at.is_some() => {
                self.probe_at = None;
                self.status = self.connected_status();
                cx.notify();
            }
            _ => {}
        }
    }
}

async fn run(
    this: gpui::WeakEntity<Organizer>,
    library: Entity<Library>,
    fetcher: Arc<Fetcher>,
    mut wake: mpsc::UnboundedReceiver<()>,
    cx: &mut AsyncApp,
) {
    let database = cx.update(|cx| library.read(cx).database());
    requeue_interrupted(&database, cx).await;
    let mut running = FuturesUnordered::new();
    loop {
        let Ok((work, take)) = this.update(cx, |this, cx| {
            let work = Arc::new(Work::new(
                &database,
                &fetcher,
                this.provider.clone(),
                this.generation.ticket(),
                cx,
            ));
            (work, this.jobs_to_take_now(running.len()))
        }) else {
            return;
        };
        if take > 0 {
            let jobs = take_jobs(&database, take, cx).await;
            running.extend(
                jobs.into_iter()
                    .map(|job| process(job, work.clone()).boxed()),
            );
            if !running.is_empty() {
                let () = library.update(cx, Library::refresh_soon);
            }
        }

        if running.is_empty() {
            let wait = match this.read_with(cx, |this, _| this.probe_at) {
                Ok(Some(probe_at)) => probe_at.saturating_duration_since(Instant::now()),
                Ok(None) => next_wait(&database, cx).await,
                Err(_) => return,
            };
            futures::select! {
                _ = wake.next() => {}
                () = cx.background_executor().timer(wait).fuse() => {}
            }
            continue;
        }

        futures::select! {
            outcome = running.select_next_some() => {
                let _ = this.update(cx, |this, cx| this.job_ended(&outcome, cx));
                let () = library.update(cx, Library::refresh_soon);
            }
            _ = wake.next() => {}
        }
    }
}

/// Jobs that were running when the app quit go back to the queue.
async fn requeue_interrupted(database: &Arc<Database>, cx: &AsyncApp) {
    let database = database.clone();
    let requeued = cx.background_spawn(async move { database.write().requeue_interrupted() });
    if let Err(err) = requeued.await {
        tracing::error!("can't requeue interrupted jobs: {err}");
    }
}

async fn take_jobs(database: &Arc<Database>, limit: usize, cx: &AsyncApp) -> Vec<Job> {
    let database = database.clone();
    let jobs = cx.background_spawn(async move { database.write().take_jobs(now(), limit) });
    jobs.await.unwrap_or_else(|err| {
        tracing::error!("can't read the organizing queue: {err}");
        Vec::new()
    })
}

/// How many jobs to start, with `free` slots and `in_flight` jobs running. While paused,
/// only one probe at a time and only once its time has come.
fn jobs_to_take(free: usize, in_flight: usize, probe_at: Option<Instant>, now: Instant) -> usize {
    match probe_at {
        None => free,
        Some(at) if in_flight == 0 && now >= at => 1,
        Some(_) => 0,
    }
}

/// How long to sleep when nothing is running: until the next retry, or an hour.
async fn next_wait(database: &Arc<Database>, cx: &AsyncApp) -> Duration {
    let database = database.clone();
    let next = cx
        .background_spawn(async move { database.write().next_job_at() })
        .await
        .ok()
        .flatten();
    match next {
        Some(at) => Duration::from_secs((at - now()).clamp(1, 3600) as u64),
        None => Duration::from_secs(3600),
    }
}

pub fn now() -> i64 {
    jiff::Timestamp::now().as_second()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paused_queue_sends_one_probe_at_a_time_and_only_when_due() {
        let now = Instant::now();
        let later = now + PROBE_EVERY;
        assert_eq!(jobs_to_take(4, 0, Some(later), now), 0);
        assert_eq!(jobs_to_take(4, 0, Some(now), now), 1);
        assert_eq!(jobs_to_take(3, 1, Some(now), now), 0);
        assert_eq!(jobs_to_take(3, 1, None, now), 3);
    }
}
