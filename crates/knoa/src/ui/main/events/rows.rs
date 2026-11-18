//! What the list and the Upcoming panel draw alike: an event row's selectable frame, and
//! the headings and notes between rows.

use gpui::{
    AnyElement, ClickEvent, Context, Div, ElementId, FontWeight, InteractiveElement as _,
    IntoElement, ParentElement as _, Pixels, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};

use super::{
    lines::{Line, Placed},
    model::Model,
};
use crate::ui::{
    main::MainView,
    theme::{ROW_RADIUS, SERIF, Theme},
};

/// How a column draws its rows: their inner side padding and how far a past one fades.
#[derive(Debug, Clone, Copy)]
pub struct RowStyle {
    pub inset: Pixels,
    pub past_opacity: f32,
}

impl MainView {
    /// The row for `event`, with `content` inside: it selects on click, opens on a double
    /// click and has the bookmark's menu, like a card. `id` is unique to this occurrence:
    /// one event can be listed twice in a column.
    pub(super) fn event_row(
        &self,
        id: ElementId,
        model: &Model,
        event: usize,
        style: RowStyle,
        cx: &mut Context<'_, MainView>,
    ) -> Stateful<Div> {
        let theme = Theme::get(cx);
        let bookmark = model.bookmark(event).id;
        let selected = self.selected == Some(bookmark);
        let hover = theme.hover;
        div()
            .id(id)
            .size_full()
            .px(style.inset)
            .flex()
            .items_center()
            .rounded(ROW_RADIUS)
            .cursor_pointer()
            .when(selected, |s| s.bg(theme.selected_row()))
            .when(!selected, |s| s.hover(move |s| s.bg(hover)))
            .when(model.is_past(event), |s| s.opacity(style.past_opacity))
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.click_card(bookmark, event, window, cx);
            }))
            .on_aux_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                this.click_card(bookmark, event, window, cx);
            }))
    }
}

/// `element` at its line's place in the column.
pub fn placed(placed: &Placed, element: AnyElement) -> Div {
    div()
        .absolute()
        .top(px(placed.y))
        .left_0()
        .right_0()
        .h(px(placed.height))
        .child(element)
}

/// A line that isn't an event: a month, a heading or a note.
pub fn text_line(line: &Line, inset: Pixels, theme: &Theme) -> AnyElement {
    match line {
        Line::Month(text) => month(text.clone(), inset, theme).into_any_element(),
        Line::Heading(text) => heading(text.clone(), inset, theme).into_any_element(),
        Line::Note(text) => note(text.clone(), inset, theme).into_any_element(),
        Line::Event(_) => div().into_any_element(),
    }
}

/// A month's name in the list, in the serif of the page titles.
fn month(text: SharedString, inset: Pixels, theme: &Theme) -> impl IntoElement {
    div()
        .size_full()
        .px(inset)
        .pt_4()
        .pb_1()
        .flex()
        .items_end()
        .font_family(SERIF)
        .text_xl()
        .text_color(theme.text)
        .child(text)
}

/// A day's or a group's name over its rows.
fn heading(text: SharedString, inset: Pixels, theme: &Theme) -> impl IntoElement {
    div()
        .size_full()
        .px(inset)
        .pt_2()
        .flex()
        .items_center()
        .text_xs()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.muted)
        .child(text)
}

/// A sentence where a section has nothing to show.
fn note(text: SharedString, inset: Pixels, theme: &Theme) -> impl IntoElement {
    div()
        .size_full()
        .px(inset)
        .flex()
        .items_center()
        .text_sm()
        .text_color(theme.muted)
        .child(text)
}
