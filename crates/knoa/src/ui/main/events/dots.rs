//! A day's events as dots, for grid cells too narrow for chips: timed events in the
//! accent, all-day ones muted, past ones dimmed, and "+N" for the rest.

use gpui::{
    Hsla, IntoElement, ParentElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use super::{day_cell::PAST_OPACITY, model::Model};
use crate::ui::theme::Theme;

/// The most dots a day shows before "+N".
const MAX_DOTS: usize = 3;
const DOT: f32 = 6.;
const GAP: f32 = 3.;
/// A line's height, the "+N" text's, so a day with a count lines up with one without.
const LINE: f32 = 12.;

pub fn dots(model: &Model, events: &[usize], theme: &Theme) -> impl IntoElement {
    let shown = events.len().min(MAX_DOTS);
    let mut dots = div()
        .flex_none()
        .h(px(LINE))
        .flex()
        .items_center()
        .gap(px(GAP));
    for &event in &events[..shown] {
        dots = dots.child(
            div()
                .size(px(DOT))
                .rounded_full()
                .bg(dot_color(model, event, theme)),
        );
    }
    // The count wraps under the dots where the day is too narrow for both.
    div()
        .flex()
        .flex_wrap()
        .gap_x(px(GAP))
        .px_0p5()
        .child(dots)
        .when(shown < events.len(), |row| {
            row.child(
                div()
                    .text_size(px(10.))
                    .line_height(px(LINE))
                    .text_color(theme.muted)
                    .child(format!("+{}", events.len() - shown)),
            )
        })
}

fn dot_color(model: &Model, event: usize, theme: &Theme) -> Hsla {
    let all_day = model.events[event].span.is_some_and(|s| s.all_day);
    let color = if all_day { theme.muted } else { theme.accent };
    if model.is_past(event) {
        color.opacity(PAST_OPACITY)
    } else {
        color
    }
}
