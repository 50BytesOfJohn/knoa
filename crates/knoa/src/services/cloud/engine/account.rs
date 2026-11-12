//! The account side of a round: asking who the account is and what it has, and what
//! follows from a change to it.

use gpui::AsyncApp;
use knoa_core::protocol::{Account, Organizer as Who, Plan};
use knoa_store::Store;

use super::{Engine, Failure, net};
use crate::services::cloud::link;

impl Engine {
    /// Asks for the account and acts on it. Returns the account when the library is
    /// linked to another one.
    pub(super) async fn refresh_account(
        &mut self,
        cx: &mut AsyncApp,
    ) -> Result<Option<Account>, Failure> {
        let account = self.fetch_account(cx).await?;
        log_account(self.account.as_ref(), &account);
        self.this
            .update(cx, |this, cx| this.set_account(account.clone(), cx))
            .map_err(|_| Failure::Stopped)?;
        if link::ensure(self, &account, cx).await? == link::Linked::OtherAccount {
            return Ok(Some(account));
        }
        if has_new_room(self.account.as_ref(), &account) {
            let resent = self.store(cx, Store::resend_unsent).await?;
            if resent > 0 {
                tracing::info!(resent, "sending bookmarks the plan's limit held back");
            }
        }
        self.account = Some(account);
        Ok(None)
    }

    /// `/v1/me`, with the email from Clerk when the API doesn't know it yet. Asked once
    /// per connection, though the account is fetched on every pull.
    async fn fetch_account(&self, cx: &AsyncApp) -> Result<Account, Failure> {
        let api = self.api.clone();
        let known_email = self.account.as_ref().and_then(|a| a.email.clone());
        net(cx, async move {
            let mut account = api.me().await?;
            account.email = account.email.or(known_email);
            if account.email.is_none() {
                let session = api.session();
                let token = session.access_token().await?;
                account.email = knoa_sync::user_email(session.http(), session.config(), &token)
                    .await
                    .unwrap_or_default();
            }
            Ok(account)
        })
        .await
    }
}

/// Says once per connection, and on every change, what the account has and who
/// organizes it. No email: the log may be shared.
fn log_account(previous: Option<&Account>, account: &Account) {
    let same = previous.is_some_and(|previous| {
        (previous.plan, previous.organizer) == (account.plan, account.organizer)
    });
    if same {
        return;
    }
    let plan = match account.plan {
        Plan::Free => "free",
        Plan::Pro => "pro",
    };
    let organizer = match account.organizer {
        Who::Cloud => "cloud",
        Who::Device => "device",
    };
    tracing::info!(plan, organizer, "cloud account");
}

/// Whether bookmarks the plan's limit refused may get through now: the account has room
/// and has just connected, grown its limit or changed plan. Anything narrower would leave
/// them waiting; anything wider would resend them on every pull while still over.
fn has_new_room(previous: Option<&Account>, account: &Account) -> bool {
    let room = account.usage.bookmarks < account.limits.bookmarks;
    let changed = previous.is_none_or(|previous| {
        previous.limits.bookmarks < account.limits.bookmarks || previous.plan != account.plan
    });
    room && changed
}
