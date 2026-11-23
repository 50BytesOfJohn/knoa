//! The detail panel's title and description, which a double-click turns into fields to
//! edit them in place.

use gpui::{
    Context, Div, Entity, Focusable as _, FontWeight, InteractiveElement as _, IntoElement as _,
    ParentElement as _, StatefulInteractiveElement as _, Styled, Window, div, px,
};
use gpui_component::input::{Escape, Textarea, TextareaState};
use knoa_core::{Bookmark, BookmarkId};

mod edit;

pub(in crate::ui::main) use edit::HeadingEdit;
use edit::Part;

use super::{MainView, sections};
use crate::{
    i18n::t,
    ui::theme::{SERIF, Theme},
};

const TITLE_SIZE: f32 = 21.;
const TITLE_LINE: f32 = 27.;

impl MainView {
    /// The title and summary, or the fields that edit them.
    pub(super) fn detail_heading(
        &self,
        bookmark: &Bookmark,
        window: &Window,
        cx: &mut Context<'_, MainView>,
    ) -> gpui::AnyElement {
        if let Some(edit) = self.heading_edit_of(bookmark.id) {
            return self.heading_fields(edit, window, cx).into_any_element();
        }
        let theme = Theme::get(cx).clone();
        // A bookmark in Trash is out of the library, like its tags and folders.
        let editable = bookmark.deleted_at.is_none();
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.editable(
                "detail-title",
                title_text(div(), &theme).child(bookmark.title.clone()),
                editable.then_some((bookmark.id, Part::Title)),
                &theme,
                cx,
            ))
            .children(sections::summary(bookmark, &theme).map(|summary| {
                self.editable(
                    "detail-summary",
                    summary,
                    editable.then_some((bookmark.id, Part::Summary)),
                    &theme,
                    cx,
                )
            }))
            .into_any_element()
    }

    /// Selects the bookmark and opens its title for editing in the detail panel.
    pub(in crate::ui::main) fn edit_title_of(
        &mut self,
        bookmark: BookmarkId,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.select(bookmark, cx);
        self.edit_heading(bookmark, Part::Title, window, cx);
    }

    /// Text a double-click edits. The hover tint, out past the text's edges so the text
    /// itself doesn't move, says it can be.
    fn editable(
        &self,
        id: &'static str,
        text: Div,
        target: Option<(BookmarkId, Part)>,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> gpui::AnyElement {
        let Some((bookmark, part)) = target else {
            return text.into_any_element();
        };
        let hover = theme.soft;
        div()
            .id(id)
            .mx(px(-6.))
            .px(px(6.))
            .rounded(px(6.))
            .hover(move |s| s.bg(hover))
            .child(text)
            .on_click(
                cx.listener(move |this, event: &gpui::ClickEvent, window, cx| {
                    if event.standard_click() && event.click_count() == 2 {
                        this.edit_heading(bookmark, part, window, cx);
                    }
                }),
            )
            .into_any_element()
    }

    fn heading_fields(
        &self,
        edit: &HeadingEdit,
        window: &Window,
        cx: &mut Context<'_, MainView>,
    ) -> Div {
        let theme = Theme::get(cx).clone();
        // The field with the caret gets the accent ring, as a focused field does on macOS.
        // The text style goes on the field itself: it sizes its rows by its own font.
        let field = |input: &Entity<TextareaState>, text: &dyn Fn(Textarea) -> Textarea| {
            let focused = input.read(cx).focus_handle(cx).is_focused(window);
            div()
                .mx(px(-6.))
                .px(px(6.))
                .py(px(2.))
                .rounded(px(6.))
                .border_1()
                .border_color(if focused {
                    theme.accent
                } else {
                    theme.hairline
                })
                .bg(theme.soft)
                .child(text(Textarea::new(input).appearance(false)))
        };
        div()
            .flex()
            .flex_col()
            .gap_2()
            .on_action(cx.listener(|this, _: &Escape, window, cx| {
                this.cancel_heading_edit(window, cx);
            }))
            // Most of the panel takes no focus, so a click there would leave no blur behind.
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                this.save_heading_edit(window, cx);
            }))
            .child(field(&edit.title, &|text| title_text(text, &theme)))
            .child(field(&edit.summary, &|text| summary_text(text, &theme)))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.faint)
                    .child(t!("detail-edit-hint")),
            )
    }
}

fn title_text<S: Styled>(base: S, theme: &Theme) -> S {
    base.font_family(SERIF)
        .text_size(px(TITLE_SIZE))
        .line_height(px(TITLE_LINE))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.text)
}

fn summary_text<S: Styled>(base: S, theme: &Theme) -> S {
    base.text_sm().line_height(px(21.)).text_color(theme.text)
}
