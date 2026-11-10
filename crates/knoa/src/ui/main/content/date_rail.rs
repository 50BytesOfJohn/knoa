//! The date rail beside the cards: one line per date group, click to scroll to it.

use gpui::{
    Context, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, px,
};

use super::cards_top;
use crate::ui::{main::MainView, theme::Theme};

impl MainView {
    /// One line per date group; longer lines for busier periods. Hover shows the label.
    pub(super) fn date_rail(
        &self,
        viewport_height: f32,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let groups = &self.layout.groups;
        let max = groups.iter().map(|g| g.len).max().unwrap_or(1) as f32;
        let scroll_top = -f32::from(self.scroll.offset().y);
        let current = self
            .layout
            .layout
            .headers
            .iter()
            .rposition(|y| *y <= scroll_top + 40.)
            .unwrap_or(0);
        let mut rail = div().flex().flex_col().items_end().gap_2();
        for (index, group) in groups.iter().enumerate().filter(|_| groups.len() > 1) {
            let length = 8. + 18. * (group.len as f32 / max).sqrt();
            let name = format!("rail-{index}");
            let target = self.layout.layout.headers[index];
            rail = rail.child(
                div()
                    .id(("rail", index))
                    .group(name.clone())
                    .relative()
                    .h(px(10.))
                    .w(px(30.))
                    .flex()
                    .items_center()
                    .justify_end()
                    .cursor_pointer()
                    .child(
                        div()
                            .h(px(2.))
                            .w(px(length))
                            .rounded_full()
                            .bg(if index == current {
                                theme.text
                            } else {
                                theme.faint.opacity(0.6)
                            })
                            .group_hover(name.clone(), |s| s.w(px(length + 8.)).bg(theme.text)),
                    )
                    .child(
                        div()
                            .absolute()
                            .right(px(36.))
                            .top(px(-7.))
                            .px_2()
                            .h(px(24.))
                            .flex()
                            .items_center()
                            .whitespace_nowrap()
                            .rounded_full()
                            .bg(theme.panel)
                            .border_1()
                            .border_color(theme.hairline)
                            .text_xs()
                            .invisible()
                            .group_hover(name, gpui::Styled::visible)
                            .child(format!("{} · {}", group.label, group.len)),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let top = cards_top() + target - 8.;
                        this.scroll
                            .set_offset(gpui::point(px(0.), px(-top.max(0.))));
                        cx.notify();
                    })),
            );
        }
        div()
            .absolute()
            .right_0()
            .top(px(viewport_height / 2. - 120.))
            .w(px(36.))
            .child(rail)
    }
}
