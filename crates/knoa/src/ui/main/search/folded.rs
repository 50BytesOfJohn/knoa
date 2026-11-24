//! The popover behind "+N": the folded tokens, each with its ×, and Clear all.

use gpui::{
    Context, Div, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, px,
};

use super::{
    MainView,
    labels::{Chip, token_chip},
    tokens::slot_id,
};
use crate::{
    i18n::t,
    ui::{components::icon, theme::Theme},
};

impl MainView {
    /// The popover behind "+N": each folded token with its ×, and Clear all.
    pub(in crate::ui::main) fn folded_tokens_popover(
        &self,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let tokens = self.library.read(cx).search().tokens.clone();
        let rows = tokens
            .iter()
            .enumerate()
            .take(self.pill.folded)
            .map(|(index, token)| self.folded_row(&token_chip(index, token), &theme, cx));
        div()
            .w(px(240.))
            .p(px(6.))
            .flex()
            .flex_col()
            .children(rows)
            .child(
                div()
                    .id("folded-clear-all")
                    .h(px(32.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .rounded(px(9.))
                    .text_sm()
                    .text_color(theme.muted)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .child(t!("main-token-clear-all"))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.set_overlay(None, cx);
                        this.clear_search(window, cx);
                    })),
            )
    }

    fn folded_row(&self, chip: &Chip, theme: &Theme, cx: &mut Context<'_, MainView>) -> Div {
        let slot = chip.slot;
        let mark = chip.mark.render(px(14.), theme);
        div()
            .h(px(32.))
            .px(px(10.))
            .flex()
            .items_center()
            .gap(px(10.))
            .text_sm()
            .child(mark)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .child(chip.label.clone()),
            )
            .child(
                div()
                    .id(("folded-remove", slot_id(slot)))
                    .size(px(22.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .child(icon("x", px(12.), theme.muted))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.remove_token(slot, window, cx);
                        if this.pill.folded <= 1 {
                            this.set_overlay(None, cx);
                        }
                    })),
            )
    }
}
