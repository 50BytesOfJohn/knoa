//! Checking the AI key pasted during setup, and why a key didn't pass.

use gpui::{AppContext as _, Context, Task, Window};
use knoa_ai::ProviderKind;

use super::{Onboarding, Step};
use crate::{i18n::t, services::ai};

/// Where the key check stands.
pub(super) enum KeyCheck {
    Idle,
    /// Holds the running check; dropping it cancels the check.
    Checking(#[allow(dead_code)] Task<()>),
    Failed(KeyProblem),
    Passed,
}

/// Why the key didn't pass, kept as data so it shows in the language of the moment.
pub(super) enum KeyProblem {
    Missing,
    Unknown,
    CheckFailed(String),
}

impl KeyProblem {
    pub(super) fn message(&self) -> String {
        match self {
            KeyProblem::Missing => t!("onboarding-ai-key-missing"),
            KeyProblem::Unknown => t!("onboarding-ai-key-unknown"),
            KeyProblem::CheckFailed(error) => {
                t!("onboarding-ai-key-check-failed", error = error.as_str())
            }
        }
    }
}

impl Onboarding {
    /// Checks the pasted key, then moves on.
    pub(super) fn check_key(&mut self, window: &mut Window, cx: &mut Context<'_, Onboarding>) {
        let key = self.key.read(cx).value().trim().to_owned();
        if key.is_empty() {
            self.key_check = KeyCheck::Failed(KeyProblem::Missing);
            cx.notify();
            return;
        }
        let check = ai::check_key(ProviderKind::OpenRouter, key, cx);
        let window_handle = window.window_handle();
        self.key_check = KeyCheck::Checking(cx.spawn(async move |this, cx| {
            let result = check.await;
            let _ = cx.update_window(window_handle, |_, window, cx| {
                let _ = this.update(cx, |this, cx| match result {
                    Ok(()) => {
                        this.key_check = KeyCheck::Passed;
                        this.go(Step::Library, window, cx);
                    }
                    Err(err) => {
                        this.key_check = KeyCheck::Failed(match err {
                            knoa_ai::Error::Unauthorized => KeyProblem::Unknown,
                            other => KeyProblem::CheckFailed(other.to_string()),
                        });
                        cx.notify();
                    }
                });
            });
        }));
        cx.notify();
    }
}
