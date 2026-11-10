//! The browser step: which profiles to bring bookmarks in from.

use gpui::{
    Context, Div, FontWeight, InteractiveElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use super::{lead, nav, title};
use crate::ui::{
    components::{blocked_browsers_notice, icon},
    onboarding::Onboarding,
    theme::Theme,
};

impl Onboarding {
    pub(super) fn browser(&mut self, cx: &mut Context<'_, Onboarding>) -> Div {
        let theme = Theme::get(cx).clone();
        let mut list = div().flex().flex_col().gap_2();
        for (index, (profile, count)) in self.profiles.iter().enumerate() {
            let id = profile.id.to_string();
            let on = self.chosen.contains(&id);
            let name = if profile.name == profile.id.dir {
                profile.id.browser.name().to_owned()
            } else {
                format!("{} · {}", profile.id.browser.name(), profile.name)
            };
            list = list.child(
                div()
                    .id(("profile", index))
                    .p_3()
                    .flex()
                    .items_center()
                    .gap_3()
                    .rounded(px(14.))
                    .border_1()
                    .border_color(if on { theme.accent } else { theme.hairline })
                    .bg(if on {
                        theme.accent.opacity(0.06)
                    } else {
                        theme.panel
                    })
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.soft))
                    .child(icon(
                        "globe",
                        px(20.),
                        if on { theme.accent } else { theme.muted },
                    ))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(div().text_sm().font_weight(FontWeight::MEDIUM).child(name))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted)
                                    .child(format!("{count} bookmarks")),
                            ),
                    )
                    .child(
                        div()
                            .size(px(20.))
                            .rounded_full()
                            .border_1()
                            .border_color(if on { theme.accent } else { theme.faint })
                            .bg(if on { theme.accent } else { theme.panel })
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(on, |s| s.child(icon("check", px(13.), theme.on_accent))),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(at) = this.chosen.iter().position(|c| *c == id) {
                            this.chosen.remove(at);
                        } else {
                            this.chosen.push(id.clone());
                        }
                        cx.notify();
                    })),
            );
        }
        if self.profiles.is_empty() && self.blocked.is_empty() {
            list = list.child(
                div().p_4().rounded(px(14.)).bg(theme.soft).text_sm().text_color(theme.muted).child(
                    "I couldn't find a Chromium-based browser here (Helium, Chrome, Brave, Edge, Vivaldi). No worries: you can pick one later in Settings.",
                ),
            );
        }
        let total: usize = self
            .profiles
            .iter()
            .filter(|(p, _)| self.chosen.contains(&p.id.to_string()))
            .map(|(_, n)| n)
            .sum();
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(title("Where do your bookmarks live?"))
            .child(lead("Keep bookmarking the way you do. I'll notice new ones within a couple of seconds. I only read; I never touch your browser's bookmarks.", &theme))
            .child(list)
            .children(blocked_browsers_notice(&self.blocked, cx))
            .child(nav(self.step, if total > 0 { format!("Bring in {total}") } else { "Next".to_owned() }, cx))
    }
}
