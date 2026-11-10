//! The last step: what came in, and the offer to organize existing bookmarks.

use gpui::{
    Context, Div, ParentElement as _, StatefulInteractiveElement as _, Styled as _, div, px,
};

use super::{lead, title};
use crate::{
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
            (_, true) => "No browser yet, so your library starts empty. Pick one in Settings whenever you like.".to_owned(),
            (0, false) => "Bringing your bookmarks in…".to_owned(),
            (n, false) => format!("{n} bookmarks are in, searchable right away."),
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
            .child(title("All set."))
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
                    .child(div().text_sm().child(format!(
                        "New bookmarks get organized on their own. Want me to go through the {} you already have too? About ${cost:.2}.",
                        counts.not_organized
                    )))
                    .child(div().flex().justify_center().child(button("organize-existing", "Yes, organize them", Tone::Soft, cx).on_click(cx.listener(|this, _, _, cx| {
                        this.library.update(cx, |library, cx| {
                            let workspace = library.workspace_id();
                            library.change(cx, move |store| store.queue_not_organized(workspace).map(drop));
                        });
                        Organizer::wake(cx);
                        this.finish(cx);
                    })))),
            );
        }
        body.child(
            button("open-library", "Open my library", Tone::Primary, cx)
                .px_6()
                .on_click(cx.listener(|this, _, _, cx| this.finish(cx))),
        )
    }
}
