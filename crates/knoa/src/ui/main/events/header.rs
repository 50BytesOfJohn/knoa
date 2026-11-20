//! The Events page's header: its title, the month and the buttons that move it in the
//! calendar, and the switch between the calendar and the list.

use gpui::{
    AnyElement, Context, Div, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use gpui_component::tooltip::Tooltip;
use jiff::Zoned;

use super::{NextMonth, PreviousMonth, ShowToday, labels, set_events_view};
use crate::{
    i18n::t,
    state::settings::EventsView,
    ui::{
        components::{Tone, button, icon},
        main::MainView,
        theme::{SERIF, Theme},
    },
};

/// The header and the space under it.
pub const HEIGHT: f32 = 52.;
/// Narrower than this, as beside the detail panel in a small window, the header leaves
/// out its title and Today (still on Cmd-T) and shortens the month, so the switch fits.
const FULL_WIDTH: f32 = 520.;

const VIEWS: [EventsView; 2] = [EventsView::Calendar, EventsView::List];

fn view_label(view: EventsView) -> String {
    match view {
        EventsView::Calendar => t!("events-view-calendar"),
        EventsView::List => t!("events-view-list"),
    }
}

/// The two-way switch between the calendar and the list. It saves the choice.
pub fn view_switch(id: &'static str, current: EventsView, theme: &Theme) -> Div {
    let mut switch = div()
        .flex()
        .p(px(2.))
        .gap(px(2.))
        .rounded_full()
        .bg(theme.soft);
    for (index, view) in VIEWS.into_iter().enumerate() {
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
            .child(view_label(view))
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
        width: f32,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let full = width >= FULL_WIDTH;
        let controls = div()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .flex()
            .items_center()
            .map(|row| if full { row.gap_3() } else { row.gap_1() })
            .when(full, |row| {
                row.child(
                    div()
                        .flex_none()
                        .font_family(SERIF)
                        .text_2xl()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(t!("events-title")),
                )
            })
            .when(view == EventsView::Calendar, |row| {
                row.children(self.month_controls(full, &theme, cx))
            });
        div()
            .flex_none()
            .h(px(HEIGHT - 12.))
            .mb(px(12.))
            .flex()
            .items_center()
            .gap_3()
            .child(controls)
            .child(view_switch("events-view", view, &theme).flex_none())
    }

    /// The month shown, the buttons to the months around it, and Today when `full`.
    fn month_controls(
        &self,
        full: bool,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> Vec<AnyElement> {
        let page_month = self.events.month;
        let month = div().text_color(theme.muted).map(|s| {
            if full {
                s.text_lg()
                    .flex_none()
                    .ml_2()
                    .min_w(px(150.))
                    .child(labels::month_heading(page_month))
            } else {
                // Shrinks before the buttons do, so both stay whole.
                s.text_base()
                    .mr_2()
                    .min_w_0()
                    .truncate()
                    .child(labels::short_month_heading(page_month, Zoned::now().date()))
            }
        });
        let previous = month_button(
            "previous-month",
            "chevron-left",
            t!("events-previous-month"),
            theme,
        )
        .on_click(cx.listener(|this, _, window, cx| {
            this.previous_month(&PreviousMonth, window, cx);
        }));
        let next = month_button(
            "next-month",
            "chevron-right",
            t!("events-next-month"),
            theme,
        )
        .on_click(cx.listener(|this, _, window, cx| this.next_month(&NextMonth, window, cx)));
        let mut controls = vec![
            month.into_any_element(),
            previous.into_any_element(),
            next.into_any_element(),
        ];
        if full {
            let today = button("events-today", t!("events-today"), Tone::Soft, cx)
                .h(px(28.))
                .px_3()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.show_today(&ShowToday, window, cx);
                }));
            controls.push(today.into_any_element());
        }
        controls
    }
}

fn month_button(
    id: &'static str,
    glyph: &'static str,
    tip: String,
    theme: &Theme,
) -> gpui::Stateful<Div> {
    let (bg, fg, border, hover) = (theme.panel, theme.text, theme.hairline, theme.hover);
    div()
        .id(id)
        .flex_none()
        .size(px(28.))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(icon(glyph, px(16.), theme.muted))
        .tooltip(move |window, cx| {
            Tooltip::new(tip.clone())
                .bg(bg)
                .text_color(fg)
                .border_color(border)
                .build(window, cx)
        })
}
