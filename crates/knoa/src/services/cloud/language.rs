//! The account's language and this Mac's. Signed in, the account's is the one that counts:
//! this Mac adopts it and gives it its own when the account has none yet. A language the
//! user picks here is the exception: it's kept as unsent in settings until the account
//! has it, so a failed request, being offline or quitting can't let the account's older
//! language undo the pick.

use gpui::{App, Context};
use knoa_core::{
    Language,
    protocol::{Account, Organizer as Who},
};

use super::{Cloud, Phase};
use crate::{
    i18n,
    services::network::Network,
    state::settings::{Delivery, SettingsFile},
};

/// This Mac's side of the language when a decision is due.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Here {
    language: Language,
    /// Unsent when the user picked `language` here and the account doesn't have it yet.
    delivery: Delivery,
    /// A request to set the account's language is on its way.
    sending: bool,
    /// Bumped on every pick and every answer to a request to set the account's language.
    epoch: u64,
}

/// What to do about the account's language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// Switch this Mac to the account's language.
    Adopt(Language),
    /// Give the account this language.
    Send(Language),
    Wait,
}

/// What follows the server answering a request to set `sent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Answered {
    delivery: Delivery,
    /// A newer pick made while `sent` was on its way, to send now.
    resend: Option<Language>,
}

/// Acts on the language of an account read in epoch `read_in`. One request at a time,
/// and the user's unsent pick beats whatever the account says, as the server keeps the
/// last one it's sent. A read that started before the latest pick or answer may predate
/// them, so its language counts for nothing.
fn decide(here: Here, account: Option<Language>, read_in: u64) -> Step {
    if here.sending || read_in < here.epoch {
        return Step::Wait;
    }
    match account {
        _ if here.delivery == Delivery::Unsent => Step::Send(here.language),
        None => Step::Send(here.language),
        Some(language) if language != here.language => Step::Adopt(language),
        Some(_) => Step::Wait,
    }
}

/// A failed request leaves the pick unsent for the next account fetch to retry, rather
/// than retrying at once in a loop. Only a newer pick is sent straight away.
fn answered(sent: Language, delivered: bool, here: Here) -> Answered {
    let unsent = here.delivery == Delivery::Unsent;
    let arrived = delivered && sent == here.language;
    Answered {
        delivery: if unsent && !arrived {
            Delivery::Unsent
        } else {
            Delivery::Sent
        },
        resend: (unsent && sent != here.language).then_some(here.language),
    }
}

impl Cloud {
    /// The user picked `language` on this Mac: switch to it, and when signed in or
    /// restoring a sign-in, keep it as unsent until the account has it.
    pub fn choose_language(language: Language, cx: &mut App) {
        i18n::set_language(language, cx);
        Cloud::entity(cx).update(cx, |cloud, cx| {
            if !cloud.signed_in_or_restoring() {
                return;
            }
            cloud.language_epoch += 1;
            set_delivery(Delivery::Unsent, cx);
            let account = cloud.account().and_then(|account| account.language);
            cloud.follow(account, cloud.language_epoch, cx);
        });
    }

    /// Whether the cloud organizes and its account doesn't have the language picked here
    /// yet. The cloud organizer writes in the account's language, so organizing again
    /// would still write in the old one.
    pub fn organizer_waits_for_language(&self, cx: &App) -> bool {
        let cloud_organizes = self
            .account()
            .is_some_and(|account| account.organizer == Who::Cloud);
        cloud_organizes && SettingsFile::get(cx).language_delivery == Delivery::Unsent
    }

    /// Credentials are present: an account exists even before `/v1/me` names it.
    fn signed_in_or_restoring(&self) -> bool {
        self.restoring || matches!(self.phase, Phase::SignedIn(_) | Phase::OtherAccount(_))
    }

    /// Acts on the language of an account the server answered with, read in epoch
    /// `read_in`.
    pub(super) fn follow_account_language(
        &mut self,
        account: &Account,
        read_in: u64,
        cx: &mut Context<'_, Cloud>,
    ) {
        self.follow(account.language, read_in, cx);
    }

    fn follow(&mut self, account: Option<Language>, read_in: u64, cx: &mut Context<'_, Cloud>) {
        match decide(self.here(cx), account, read_in) {
            Step::Adopt(language) => i18n::set_language(language, cx),
            Step::Send(language) => self.send_language(language, cx),
            Step::Wait => {}
        }
    }

    fn here(&self, cx: &App) -> Here {
        let settings = SettingsFile::get(cx);
        let sending = matches!(
            &self.phase,
            Phase::SignedIn(connection) if connection.language_push.is_some()
        );
        Here {
            language: settings.language,
            delivery: settings.language_delivery,
            sending,
            epoch: self.language_epoch,
        }
    }

    /// `PUT /v1/me/language`. Signed out, or linked to another account, there's no
    /// account to tell yet.
    fn send_language(&mut self, language: Language, cx: &mut Context<'_, Cloud>) {
        let Phase::SignedIn(connection) = &mut self.phase else {
            return;
        };
        let api = connection.api.clone();
        let generation = self.generation.ticket();
        let task = Network::spawn(cx, async move { api.set_language(language).await });
        connection.language_push = Some(cx.spawn(async move |this, cx| {
            let result = match task.await {
                Ok(result) => result.map_err(anyhow::Error::from),
                Err(err) => Err(anyhow::Error::from(err)),
            };
            let _ = this.update(cx, |this, cx| {
                if generation.if_current(|| ()).is_some() {
                    this.language_answered(language, result, cx);
                }
            });
        }));
    }

    fn language_answered(
        &mut self,
        sent: Language,
        result: anyhow::Result<Account>,
        cx: &mut Context<'_, Cloud>,
    ) {
        if let Phase::SignedIn(connection) = &mut self.phase {
            connection.language_push = None;
        }
        self.language_epoch += 1;
        if let Err(err) = &result {
            tracing::warn!("can't set the account's language: {err}");
        }
        let next = answered(sent, result.is_ok(), self.here(cx));
        set_delivery(next.delivery, cx);
        if let Some(language) = next.resend {
            self.send_language(language, cx);
        }
        if let Ok(account) = result {
            self.set_account(account, self.language_epoch, cx);
        }
    }
}

/// Saved only on a change: settings are written to disk on every update.
pub(super) fn set_delivery(delivery: Delivery, cx: &mut App) {
    if SettingsFile::get(cx).language_delivery != delivery {
        SettingsFile::update(cx, |s| s.language_delivery = delivery);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use Delivery::{Sent, Unsent};

    fn here(language: Language, delivery: Delivery, sending: bool) -> Here {
        Here {
            language,
            delivery,
            sending,
            epoch: 0,
        }
    }

    #[test]
    fn an_account_language_is_adopted_and_a_missing_one_gets_this_macs() {
        let settled = here(Language::En, Sent, false);
        assert_eq!(
            decide(settled, Some(Language::Pl), 0),
            Step::Adopt(Language::Pl)
        );
        assert_eq!(decide(settled, Some(Language::En), 0), Step::Wait);
        assert_eq!(decide(settled, None, 0), Step::Send(Language::En));
    }

    #[test]
    fn an_unsent_pick_beats_the_account_language() {
        let picked = here(Language::Pl, Unsent, false);
        assert_eq!(
            decide(picked, Some(Language::En), 0),
            Step::Send(Language::Pl)
        );
        let on_its_way = here(Language::Pl, Unsent, true);
        assert_eq!(decide(on_its_way, Some(Language::En), 0), Step::Wait);
    }

    #[test]
    fn an_account_read_from_before_the_last_pick_or_answer_keeps_this_macs_language() {
        // Picked Polish and the account took it, while a read from before still said English.
        let delivered = Here {
            epoch: 2,
            ..here(Language::Pl, Sent, false)
        };
        assert_eq!(decide(delivered, Some(Language::En), 1), Step::Wait);
        assert_eq!(
            decide(delivered, Some(Language::En), 2),
            Step::Adopt(Language::En)
        );
    }

    #[test]
    fn picks_made_while_one_is_sent_coalesce_and_the_newest_wins() {
        // Picked German, then French and Polish while German was on its way.
        let now = here(Language::Pl, Unsent, false);
        let after_german = answered(Language::De, true, now);
        assert_eq!(
            after_german,
            Answered {
                delivery: Unsent,
                resend: Some(Language::Pl)
            }
        );
        let after_polish = answered(Language::Pl, true, now);
        assert_eq!(
            after_polish,
            Answered {
                delivery: Sent,
                resend: None
            }
        );
    }

    #[test]
    fn a_failed_request_keeps_the_pick_unsent_without_retrying_at_once() {
        let now = here(Language::Pl, Unsent, false);
        assert_eq!(
            answered(Language::Pl, false, now),
            Answered {
                delivery: Unsent,
                resend: None
            }
        );
    }
}
