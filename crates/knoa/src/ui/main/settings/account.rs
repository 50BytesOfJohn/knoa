//! The Account page: what Cloud mode is and signing in, or the signed-in account, its
//! plan, how sync is doing, and signing out.

pub(super) mod plan;

use gpui::{
    AnyElement, Context, Div, Entity, IntoElement, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Window, div,
};
use jiff::Zoned;

use super::{form, updates::When};
use crate::{
    i18n::t,
    services::cloud::{Cloud, Notice, Phase, Problem, Rejection, SyncError, SyncStatus},
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};

pub struct AccountPage {
    cloud: Entity<Cloud>,
}

impl AccountPage {
    pub fn new(cx: &mut Context<'_, AccountPage>) -> AccountPage {
        let cloud = Cloud::entity(cx);
        cx.observe(&cloud, |_, _, cx| cx.notify()).detach();
        AccountPage { cloud }
    }
}

impl Render for AccountPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let cloud = self.cloud.read(cx);
        let header = form::header(
            t!("settings-account-title"),
            &t!("settings-account-about"),
            &theme,
        );
        let mut page = div().flex().flex_col().gap_8().child(header);
        if cloud.config().is_none() {
            let mut rows = vec![form::note(t!("settings-account-unavailable"), theme.muted)];
            if let Some(reason) = cloud.unavailable() {
                rows.push(form::problem_note(
                    t!("settings-account-couldnt-start", reason = reason),
                    &theme,
                ));
            }
            return page.child(form::group(
                &t!("settings-account-cloud-mode"),
                rows,
                &theme,
            ));
        }
        page = match cloud.phase() {
            Phase::SignedOut | Phase::SigningIn(_) => page.child(self.signed_out(&theme, cx)),
            Phase::OtherAccount(_) => page.child(self.other_account(&theme, cx)),
            Phase::SignedIn(_) => page
                .child(self.account_group(&theme, cx))
                .child(self.sync_group(&theme, cx)),
        };
        page
    }
}

impl AccountPage {
    fn signed_out(&self, theme: &Theme, cx: &mut Context<'_, AccountPage>) -> Div {
        let cloud = self.cloud.read(cx);
        let waiting = matches!(cloud.phase(), Phase::SigningIn(_));
        let mut rows = vec![
            form::note(t!("settings-account-what-it-is"), theme.text),
            form::note(t!("settings-account-privacy"), theme.muted),
        ];
        if let Some(problem) = cloud.problem() {
            rows.push(form::problem_note(problem_text(problem), theme));
        }
        rows.push(if waiting {
            form::row(
                &t!("settings-account-finish-signing-in"),
                Some(&t!("settings-account-finish-signing-in-about")),
                button(
                    "cancel-sign-in",
                    t!("settings-account-cancel"),
                    Tone::Soft,
                    cx,
                )
                .on_click(
                    cx.listener(|this, _, _, cx| this.cloud.update(cx, Cloud::cancel_sign_in)),
                ),
                theme,
            )
        } else {
            form::row(
                &t!("settings-account-knoa-account"),
                Some(&t!("settings-account-sign-in-about")),
                button("sign-in", t!("settings-account-sign-in"), Tone::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.cloud.update(cx, Cloud::sign_in))),
                theme,
            )
        });
        form::group(&t!("settings-account-cloud-mode"), rows, theme)
    }

    fn other_account(&self, theme: &Theme, cx: &mut Context<'_, AccountPage>) -> Div {
        let sync_with = match self.email(cx) {
            Some(email) => t!("settings-account-sync-with", email = email),
            None => t!("settings-account-sync-with-this-account"),
        };
        let rows = vec![
            form::problem_note(t!("settings-account-other-account"), theme),
            form::row(
                &sync_with,
                Some(&t!("settings-account-sync-with-about")),
                button(
                    "replace-account",
                    t!("settings-account-use-this-account"),
                    Tone::Primary,
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    this.cloud.update(cx, Cloud::replace_other_account);
                })),
                theme,
            ),
            self.sign_out_row(theme, cx),
        ];
        form::group(&t!("settings-account-title"), rows, theme)
    }

    fn account_group(&self, theme: &Theme, cx: &mut Context<'_, AccountPage>) -> Div {
        let cloud = self.cloud.read(cx);
        let account = cloud.account().cloned();
        let config = cloud.config().cloned();
        let plan_row = match (&account, config) {
            (Some(account), Some(config)) => {
                let about = plan::description(account);
                let email = self.email(cx);
                form::row(
                    &email.unwrap_or_else(|| t!("settings-account-this-account")),
                    Some(&about),
                    plan::button("plan", account, &config, cx),
                    theme,
                )
            }
            _ => form::note(t!("settings-account-asking"), theme.muted),
        };
        let mut rows = vec![plan_row];
        rows.extend(
            account
                .as_ref()
                .map(|account| plan::usage_row(account, theme)),
        );
        rows.push(self.sign_out_row(theme, cx));
        form::group(&t!("settings-account-title"), rows, theme)
    }

    fn sync_group(&self, theme: &Theme, cx: &mut Context<'_, AccountPage>) -> Div {
        let cloud = self.cloud.read(cx);
        let status = match cloud.sync() {
            SyncStatus::Starting => t!("settings-account-connecting"),
            SyncStatus::UpToDate => t!("settings-account-up-to-date"),
            SyncStatus::Syncing(changes) => t!("settings-account-syncing", count = (*changes)),
            SyncStatus::Offline => t!("settings-account-offline"),
            SyncStatus::Error(err) => {
                t!(
                    "settings-account-sync-stopped",
                    error = (sync_error_text(err))
                )
            }
        };
        let last = match cloud.last_synced().map(|at| When::of(at, &Zoned::now())) {
            Some(When::Today(time)) => t!("settings-account-synced-today", time = time),
            Some(When::Yesterday(time)) => t!("settings-account-synced-yesterday", time = time),
            Some(When::On(date)) => t!("settings-account-synced-on", date = date),
            None => t!("settings-account-not-synced"),
        };
        let mut rows = vec![form::row(&status, Some(&last), gpui::Empty, theme)];
        let notices = cloud.notices();
        if !notices.is_empty() {
            let text: Vec<String> = notices.iter().map(notice_text).collect();
            rows.push(form::problem_note(text.join(" "), theme));
        }
        form::group(&t!("settings-account-sync"), rows, theme)
    }

    fn sign_out_row(&self, theme: &Theme, cx: &mut Context<'_, AccountPage>) -> AnyElement {
        form::row(
            &t!("settings-account-sign-out"),
            Some(&t!("settings-account-sign-out-about")),
            button("sign-out", t!("settings-account-sign-out"), Tone::Soft, cx)
                .on_click(cx.listener(|this, _, _, cx| this.cloud.update(cx, Cloud::sign_out))),
            theme,
        )
    }

    fn email(&self, cx: &Context<'_, AccountPage>) -> Option<String> {
        self.cloud
            .read(cx)
            .account()
            .and_then(|account| account.email.clone())
    }
}

fn problem_text(problem: &Problem) -> String {
    match problem {
        Problem::SignInFailed(problem) => problem.clone(),
        Problem::SignedOut => t!("error-signed-out"),
        Problem::UnlinkFailed(error) => t!("error-unlink-library", error = error.as_str()),
    }
}

fn sync_error_text(error: &SyncError) -> String {
    match error {
        SyncError::Unanswered => t!("error-sync-unanswered"),
        SyncError::Failed(error) => error.clone(),
    }
}

fn notice_text(notice: &Notice) -> String {
    match *notice {
        Notice::PlanLimit { limit, unsent } => t!(
            "settings-account-notice-plan-limit",
            limit = (limit),
            unsent = (unsent)
        ),
        Notice::Rejected(Rejection::Invalid) => t!("settings-account-notice-rejected"),
        Notice::Rejected(Rejection::TooLarge) => t!("settings-account-notice-too-large"),
        Notice::AllowanceUsedUp => t!("settings-account-notice-allowance-used-up"),
    }
}
