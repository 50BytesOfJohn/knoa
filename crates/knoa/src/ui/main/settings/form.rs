//! The pieces every settings page is built from: a page header, groups of rows, and rows
//! with a label and description on the left and their control on the right.

use gpui::{
    AnyElement, Div, FontWeight, IntoElement, ParentElement as _, Styled as _, div,
    prelude::FluentBuilder as _, px,
};

use crate::ui::{
    components::problem,
    theme::{CARD_RADIUS, SERIF, Theme},
};

/// The page's title and one line on what it holds.
pub fn header(title: impl Into<String>, about: &str, theme: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .font_family(SERIF)
                .text_2xl()
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.into()),
        )
        .child(
            div()
                .text_sm()
                .text_color(theme.muted)
                .child(about.to_owned()),
        )
}

/// A titled block of rows, separated by hairlines.
pub fn group(title: &str, rows: Vec<AnyElement>, theme: &Theme) -> Div {
    let mut body = div()
        .flex()
        .flex_col()
        .rounded(CARD_RADIUS)
        .border_1()
        .border_color(theme.hairline);
    for (index, row) in rows.into_iter().enumerate() {
        body = body.child(
            div()
                .when(index > 0, |s| s.border_t_1().border_color(theme.hairline))
                .child(row),
        );
    }
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(group_title(title, theme))
        .child(body)
}

pub fn group_title(title: &str, theme: &Theme) -> Div {
    div()
        .px_1()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.text)
        .child(title.to_owned())
}

/// A setting with its control on the right.
pub fn row(
    label: &str,
    about: Option<&str>,
    control: impl IntoElement,
    theme: &Theme,
) -> AnyElement {
    div()
        .min_h(px(52.))
        .px_4()
        .py_3()
        .flex()
        .items_center()
        .gap_4()
        .child(label_block(label, about, theme))
        .child(div().flex_none().child(control))
        .into_any_element()
}

/// A setting whose control needs the full width, like a text area or a list.
pub fn stacked(
    label: &str,
    about: Option<&str>,
    body: impl IntoElement,
    theme: &Theme,
) -> AnyElement {
    div()
        .px_4()
        .py_3()
        .flex()
        .flex_col()
        .gap_3()
        .child(label_block(label, about, theme))
        .child(body)
        .into_any_element()
}

/// A line of text in a group, for states with nothing to set.
pub fn note(text: impl Into<String>, color: gpui::Hsla) -> AnyElement {
    div()
        .px_4()
        .py_3()
        .text_sm()
        .text_color(color)
        .child(text.into())
        .into_any_element()
}

/// A problem and how to fix it, set like a note.
pub fn problem_note(text: impl Into<String>, theme: &Theme) -> AnyElement {
    div()
        .px_4()
        .py_3()
        .child(problem(text.into(), theme))
        .into_any_element()
}

fn label_block(label: &str, about: Option<&str>, theme: &Theme) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap_0p5()
        .child(div().text_sm().child(label.to_owned()))
        .when_some(about, |s, about| {
            s.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child(about.to_owned()),
            )
        })
}
