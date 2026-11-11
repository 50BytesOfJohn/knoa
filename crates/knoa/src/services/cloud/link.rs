//! Linking the library to the signed-in account the first time: everything the account
//! has, merged with what this Mac has, in one transaction.

use std::time::Instant;

use gpui::AsyncApp;
use knoa_core::protocol::Account;

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
    let mut changes = Vec::new();
    let mut cursor = 0;
    loop {
        engine.status(SyncStatus::Syncing(changes.len()), cx)?;
        let api = engine.api.clone();
        let page = net(cx, async move { api.pull(cursor).await }).await?;
        changes.extend(page.changes);
        cursor = page.cursor;
        if !page.more {
            break;
        }
    }
    let pulled = changes.len();
    let user_id = account.user_id.clone();
    engine
        .store(cx, move |store| store.link(&user_id, &changes, cursor))
        .await?;
    tracing::info!(
        pulled,
        took = ?started.elapsed(),
        "linked the library to the cloud account"
    );
    engine.library.update(cx, Library::refresh);
    Ok(())
}
