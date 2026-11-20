//! The last step: what came in, and the offer to organize existing bookmarks.

use gpui::{
    Context, Div, ParentElement as _, StatefulInteractiveElement as _, Styled as _, div, px,
};

use super::{lead, title};
use crate::{
    format,
    i18n::t,
    services::organizer::Organizer,
    ui::{
        components::{Tone, button},
        mascot::{Mood, mascot},
        onboarding::{KeyCheck, Onboarding},
        theme::Theme,
    },
};

impl Onboarding {
    pub(super) fn done(&mut self, cx: &mut Context<'_, Onboarding>) -> Div {
        let theme = Theme::get(cx).clone();
        let counts = self.library.read(cx).counts();
        let ai_on = matches!(self.key_check, KeyCheck::Passed);
        let line = match (counts.everything, self.chosen.is_empty()) {
            (_, true) => t!("onboarding-done-no-browser"),
            (0, false) => t!("onboarding-done-bringing-in"),
            (n, false) => t!("onboarding-done-count", count = n),
        };
        let mut body = div()
            .flex()
            .flex_col()
            .items_center()
            .text_center()
            .gap_4()
            .pt_6()
            .child(mascot(
                "all-set",
                px(64.),
                Mood::Happy,
                theme.ai,
                theme.panel,
            ))
            .child(title(&t!("onboarding-done-title")))
            .child(lead(&line, &theme));
        if ai_on && counts.not_organized > 0 {
            let cost = counts.not_organized as f64 * 0.000_17;
            body = body.child(
                div()
                    .p_4()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .rounded(px(16.))
                    .bg(theme.ai_soft)
                    .child(div().text_sm().child(t!(
                        "onboarding-done-organize-existing",
                        count = counts.not_organized,
                        cost = (format::usd(cost, 2))
                    )))
                    .child(
                        div().flex().justify_center().child(
                            button(
                                "organize-existing",
                                t!("onboarding-done-organize"),
                                Tone::Soft,
                                cx,
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.library.update(cx, |library, cx| {
                                    let workspace = library.workspace_id();
                                    library.change(cx, move |store| {
                                        store.queue_not_organized(workspace).map(drop)
                                    });
                                });
                                Organizer::wake(cx);
                                this.finish(cx);
                            })),
                        ),
                    ),
            );
        }
        body.child(
            button(
                "open-library",
                t!("onboarding-done-open-library"),
                Tone::Primary,
                cx,
            )
            .px_6()
            .on_click(cx.listener(|this, _, _, cx| this.finish(cx))),
        )
    }
}
