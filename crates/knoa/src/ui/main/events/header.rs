//! The Events page's header: its title, the month and the buttons that move it in the
//! calendar, and the switch between the calendar and the list.

use gpui::{
    AnyElement, Context, Div, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use gpui_component::tooltip::Tooltip;

use super::{NextMonth, PreviousMonth, ShowToday, labels, set_events_view};
use crate::{
    state::settings::EventsView,
    ui::{
        components::{Tone, button, icon},
        main::MainView,
        theme::{SERIF, Theme},
    },
};

/// The header and the space under it.
pub const HEIGHT: f32 = 52.;

const VIEWS: [(EventsView, &str); 2] = [
    (EventsView::Calendar, "Calendar"),
    (EventsView::List, "List"),
];

/// The two-way switch between the calendar and the list. It saves the choice.
pub fn view_switch(id: &'static str, current: EventsView, theme: &Theme) -> Div {
    let mut switch = div()
        .flex()
        .p(px(2.))
        .gap(px(2.))
        .rounded_full()
        .bg(theme.soft);
    for (index, (view, label)) in VIEWS.into_iter().enumerate() {
        let (text, hover) = (theme.text, theme.hover);
        let segment = div()
            .id((id, index))
            .h(px(26.))
            .px_3()
            .flex()
            .items_center()
            .rounded_full()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .cursor_pointer()
            .map(|s| {
                if view == current {
                    s.bg(theme.panel).text_color(text).shadow_sm()
                } else {
                    s.text_color(theme.muted)
                        .hover(move |s| s.bg(hover).text_color(text))
                }
            })
            .child(label)
            .on_click(move |_, _, cx| {
                if view != current {
                    set_events_view(view, cx);
                }
            });
        switch = switch.child(segment);
    }
    switch
}

impl MainView {
    pub(super) fn events_header(
        &self,
        view: EventsView,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        div()
            .flex_none()
            .h(px(HEIGHT - 12.))
            .mb(px(12.))
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .font_family(SERIF)
                    .text_2xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Events"),
            )
            .when(view == EventsView::Calendar, |row| {
                row.children(self.month_controls(&theme, cx))
            })
            .child(div().flex_1())
            .child(view_switch("events-view", view, &theme))
    }

    /// The month shown, the buttons to the months around it, and Today.
    fn month_controls(&self, theme: &Theme, cx: &mut Context<'_, MainView>) -> [AnyElement; 4] {
        let month = div()
            .ml_2()
            .min_w(px(150.))
            .text_lg()
            .text_color(theme.muted)
            .child(labels::month_heading(self.events.month));
        let previous = month_button("previous-month", "chevron-left", "Previous month", theme)
            .on_click(cx.listener(|this, _, window, cx| {
                this.previous_month(&PreviousMonth, window, cx);
            }));
        let next = month_button("next-month", "chevron-right", "Next month", theme)
            .on_click(cx.listener(|this, _, window, cx| this.next_month(&NextMonth, window, cx)));
        let today = button("events-today", "Today", Tone::Soft, cx)
            .h(px(28.))
            .px_3()
            .on_click(cx.listener(|this, _, window, cx| {
                this.show_today(&ShowToday, window, cx);
            }));
        [
            month.into_any_element(),
            previous.into_any_element(),
            next.into_any_element(),
            today.into_any_element(),
        ]
    }
}

fn month_button(
    id: &'static str,
    glyph: &'static str,
    tip: &'static str,
    theme: &Theme,
) -> gpui::Stateful<Div> {
    let (bg, fg, border, hover) = (theme.panel, theme.text, theme.hairline, theme.hover);
    div()
        .id(id)
        .size(px(28.))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(icon(glyph, px(16.), theme.muted))
        .tooltip(move |window, cx| {
            Tooltip::new(tip)
                .bg(bg)
                .text_color(fg)
                .border_color(border)
                .build(window, cx)
        })
}
