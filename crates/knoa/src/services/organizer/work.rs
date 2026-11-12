//! Resources and guarded writes for one generation of local organizing work.

use std::sync::Arc;

use gpui::{App, BackgroundExecutor};
use knoa_ai::Provider;
use knoa_fetch::Fetcher;
use knoa_store::{Job, Store};
use tokio::runtime::Handle;

use super::job::Outcome;
use crate::{
    services::{generation::Ticket, network::Network},
    state::database::Database,
};

/// Everything one job needs, owned so it can move to other threads.
pub(super) struct Work {
    pub(super) database: Arc<Database>,
    pub(super) fetcher: Arc<Fetcher>,
    pub(super) provider: Option<Arc<Provider>>,
    pub(super) background: BackgroundExecutor,
    pub(super) network: Handle,
    ticket: Ticket,
}

impl Work {
    pub(super) fn new(
        database: &Arc<Database>,
        fetcher: &Arc<Fetcher>,
        provider: Option<Arc<Provider>>,
        ticket: Ticket,
        cx: &App,
    ) -> Work {
        Work {
            database: database.clone(),
            fetcher: fetcher.clone(),
            provider,
            background: cx.background_executor().clone(),
            network: Network::handle(cx),
            ticket,
        }
    }

    pub(super) async fn save(
        &self,
        job: &Job,
        action: impl FnOnce(&mut Store) -> knoa_store::Result<()> + Send + 'static,
    ) -> Outcome {
        let database = self.database.clone();
        let ticket = self.ticket.clone();
        let job = job.clone();
        self.background
            .spawn(async move {
                let mut store = database.write();
                match store.save_job(&job, |store| {
                    if let Some(result) = ticket.if_current(|| action(store)) {
                        result.map(|()| Outcome::Done)
                    } else {
                        store.return_interrupted_job(job.bookmark.id)?;
                        Ok(Outcome::Superseded)
                    }
                }) {
                    Ok(Some(outcome)) => outcome,
                    Ok(None) => Outcome::Superseded,
                    Err(err) => Outcome::Failed(err.to_string()),
                }
            })
            .await
    }

    pub(super) fn current(&self) -> bool {
        self.ticket.if_current(|| ()).is_some()
    }
}
