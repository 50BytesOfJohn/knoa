//! The account's language and this Mac's. Signed in, the account's is the one that counts:
//! this Mac adopts it, gives it its own when the account has none yet, and sends every
//! change the user makes here.

use gpui::{App, Context};
use knoa_core::{Language, protocol::Account};

use super::{Cloud, Phase};
use crate::{i18n, services::network::Network, state::settings::SettingsFile};

/// What an account's language means for this Mac.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Follow {
    /// Switch to the account's language.
    Adopt(Language),
    /// The account has none yet: give it this Mac's.
    Push(Language),
    Nothing,
}

fn follow(account: Option<Language>, here: Language) -> Follow {
    match account {
        None => Follow::Push(here),
        Some(language) if language != here => Follow::Adopt(language),
        Some(_) => Follow::Nothing,
    }
}

impl Cloud {
    /// The user picked `language` on this Mac: switch to it, and tell the account when
    /// signed in.
    pub fn choose_language(language: Language, cx: &mut App) {
        i18n::set_language(language, cx);
        Cloud::entity(cx).update(cx, |cloud, cx| cloud.push_language(language, cx));
    }

    /// Acts on the language of an account the server just answered with. While a change
    /// from here is on its way, an older answer would undo it, so it waits for that one.
    pub(super) fn follow_account_language(
        &mut self,
        account: &Account,
        cx: &mut Context<'_, Cloud>,
    ) {
        if let Phase::SignedIn(connection) = &self.phase
            && connection.language_push.is_some()
        {
            return;
        }
        match follow(account.language, SettingsFile::get(cx).language) {
            Follow::Adopt(language) => i18n::set_language(language, cx),
            Follow::Push(language) => self.push_language(language, cx),
            Follow::Nothing => {}
        }
    }

    /// `PUT /v1/me/language`. Signed out, there's no account to tell.
    fn push_language(&mut self, language: Language, cx: &mut Context<'_, Cloud>) {
        let Phase::SignedIn(connection) = &mut self.phase else {
            return;
        };
        let api = connection.api.clone();
        let generation = self.generation.ticket();
        let task = Network::spawn(cx, async move { api.set_language(language).await });
        connection.language_push = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                if generation.if_current(|| ()).is_none() {
                    return;
                }
                if let Phase::SignedIn(connection) = &mut this.phase {
                    connection.language_push = None;
                }
                match result {
                    Ok(Ok(account)) => this.set_account(account, cx),
                    Ok(Err(err)) => tracing::warn!("can't set the account's language: {err}"),
                    Err(err) => tracing::warn!("can't set the account's language: {err}"),
                }
            });
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_account_language_wins_and_a_missing_one_gets_this_macs() {
        assert_eq!(
            follow(Some(Language::Pl), Language::En),
            Follow::Adopt(Language::Pl)
        );
        assert_eq!(follow(None, Language::De), Follow::Push(Language::De));
        assert_eq!(follow(Some(Language::Fr), Language::Fr), Follow::Nothing);
    }
}
