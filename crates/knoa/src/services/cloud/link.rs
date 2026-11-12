//! Linking the library to the signed-in account the first time. A look at the whole
//! account without page texts tells linking which workspaces and folders to adopt; then
//! everything the account has comes in by ordinary pulls, a page at a time, so the
//! account is never held in memory whole.

use std::time::Instant;

use gpui::AsyncApp;
use knoa_core::protocol::Account;
use knoa_store::AccountRows;

use super::{
    SyncStatus,
    engine::{Engine, Failure, net},
};
use crate::state::library::Library;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Linked {
    Yes,
    /// The library belongs to another account; the user has to choose.
    OtherAccount,
}

pub(super) async fn ensure(
    engine: &Engine,
    account: &Account,
    cx: &mut AsyncApp,
) -> Result<Linked, Failure> {
    match engine.store(cx, |store| store.account()).await? {
        Some(linked) if linked == account.user_id => Ok(Linked::Yes),
        Some(_) => Ok(Linked::OtherAccount),
        None => link(engine, account, cx).await.map(|()| Linked::Yes),
    }
}

async fn link(engine: &Engine, account: &Account, cx: &mut AsyncApp) -> Result<(), Failure> {
    let started = Instant::now();
    let rows = account_rows(engine, cx).await?;
    let user_id = account.user_id.clone();
    engine
        .store(cx, move |store| store.link(&user_id, rows))
        .await?;
    engine.library.update(cx, Library::refresh);
    // Before the first push, as linking always did; an interrupted pull carries on from
    // its cursor in the next round.
    engine.pull(cx).await?;
    tracing::info!(took = ?started.elapsed(), "linked the library to the cloud account");
    Ok(())
}

/// What linking matches against, pulled from cursor 0 without page texts.
async fn account_rows(engine: &Engine, cx: &mut AsyncApp) -> Result<AccountRows, Failure> {
    let mut rows = AccountRows::default();
    let mut cursor = 0;
    let mut seen = 0;
    loop {
        engine.status(SyncStatus::Syncing(seen), cx)?;
        let api = engine.api.clone();
        let page = net(cx, async move { api.pull_without_texts(cursor).await }).await?;
        seen += page.changes.len();
        rows.add(&page.changes);
        cursor = page.cursor;
        if !page.more {
            return Ok(rows);
        }
    }
}
