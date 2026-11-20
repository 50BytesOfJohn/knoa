//! Connecting an OpenRouter key on the AI page, and how the check went.

use gpui::{
    AnyElement, Context, Div, ParentElement as _, StatefulInteractiveElement as _, Styled as _,
    TaskExt as _, div,
};
use gpui_component::input::Input;
use knoa_ai::ProviderKind;

use super::{super::form, AiPage};
use crate::{
    i18n::t,
    services::ai,
    ui::{
        components::{Tone, button},
        theme::Theme,
    },
};

pub(super) enum KeyCheck {
    Checking,
    Connected,
    Failed(String),
}

impl AiPage {
    pub(super) fn connect_group(&self, theme: &Theme, cx: &mut Context<'_, AiPage>) -> Div {
        let field = div()
            .flex()
            .gap_2()
            .child(div().flex_1().child(Input::new(&self.key)))
            .child(
                button("connect-ai", t!("settings-ai-connect"), Tone::Primary, cx)
                    .on_click(cx.listener(|this, _, _, cx| this.connect(cx))),
            );
        let mut rows = vec![form::stacked(
            &t!("settings-ai-key"),
            Some(&t!("settings-ai-key-about")),
            field,
            theme,
        )];
        rows.extend(self.message_row(theme));
        rows.push(form::row(
            &t!("settings-ai-no-key"),
            Some(&t!("settings-ai-no-key-about")),
            button("get-key", t!("settings-ai-get-key"), Tone::Soft, cx)
                .on_click(|_, _, cx| cx.open_url(ProviderKind::OpenRouter.keys_url())),
            theme,
        ));
        form::group(&t!("settings-ai-provider"), rows, theme)
    }

    fn connect(&mut self, cx: &mut Context<'_, AiPage>) {
        let key = self.key.read(cx).value().trim().to_owned();
        if key.is_empty() {
            return;
        }
        self.key_check = Some(KeyCheck::Checking);
        let check = ai::check_key(ProviderKind::OpenRouter, key.clone(), cx);
        self.checking = Some(cx.spawn(async move |this, cx| {
            let result = check.await;
            let _ = this.update(cx, |this, cx| {
                this.key_check = Some(match result {
                    Ok(()) => {
                        ai::enable(ProviderKind::OpenRouter, &key, cx).detach_and_log_err(cx);
                        KeyCheck::Connected
                    }
                    Err(err) => KeyCheck::Failed(err.to_string()),
                });
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub(super) fn message_row(&self, theme: &Theme) -> Option<AnyElement> {
        Some(match self.key_check.as_ref()? {
            KeyCheck::Checking => form::note(t!("settings-ai-checking"), theme.muted),
            KeyCheck::Connected => form::note(t!("settings-ai-key-connected"), theme.muted),
            KeyCheck::Failed(err) => {
                form::problem_note(t!("settings-ai-key-failed", error = err), theme)
            }
        })
    }
}
