//! Deleting bookmarks: moving them to Trash, restoring them, and emptying Trash.

use gpui::{
    Context, Div, FontWeight, ParentElement as _, StatefulInteractiveElement as _, Styled as _,
    Window, div, px,
};
use knoa_core::{Bookmark, BookmarkId, Place, TRASH_SECONDS};

use super::{DeleteSelected, MainView};
use crate::ui::{
    components::{Tone, button, icon},
    theme::{CARD_RADIUS, PANEL_RADIUS, Theme},
};

pub const TRASH_BAR_HEIGHT: f32 = 56.;

impl MainView {
    pub(super) fn delete_selected(
        &mut self,
        _: &DeleteSelected,
        _: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        // In Trash, deleting for good takes a click on purpose.
        if let Some(id) = self.selected
            && self.library.read(cx).place() != Place::Trash
        {
            self.move_to_trash(id, cx);
        }
    }

    /// Moves a bookmark to Trash and selects its neighbour, so ⌫ can be pressed again.
    pub(super) fn move_to_trash(&mut self, id: BookmarkId, cx: &mut Context<'_, MainView>) {
        if self.selected == Some(id) {
            let neighbour = self.neighbour_of(id);
            self.set_selected(neighbour, cx);
            self.detail_open &= self.selected.is_some();
        }
        let now = now();
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.trash(id, now));
        });
    }

    /// The strip above the cards in Trash. Emptying it asks for a second click.
    pub(super) fn trash_bar(&self, width: f32, cx: &mut Context<'_, MainView>) -> Div {
        let theme = Theme::get(cx).clone();
        let count = self.library.read(cx).results().len();
        let label = if self.confirm_empty_trash {
            format!("Delete {count} for good")
        } else {
            "Empty Trash".to_owned()
        };
        let tone = if self.confirm_empty_trash {
            Tone::Primary
        } else {
            Tone::Soft
        };
        div()
            .absolute()
            .top_0()
            .left_0()
            .w(px(width))
            .h(px(TRASH_BAR_HEIGHT))
            .px_4()
            .flex()
            .items_center()
            .gap_2()
            .rounded(PANEL_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .text_sm()
            .child(icon("trash", px(15.), theme.muted))
            .child(
                div()
                    .flex_1()
                    .text_color(theme.muted)
                    .child("Bookmarks in Trash are deleted for good after 30 days."),
            )
            .child(button("empty-trash", label, tone, cx).on_click(cx.listener(
                |this, _, _, cx| {
                    if this.confirm_empty_trash {
                        this.empty_trash(cx);
                    } else {
                        this.confirm_empty_trash = true;
                        cx.notify();
                    }
                },
            )))
    }

    fn empty_trash(&mut self, cx: &mut Context<'_, MainView>) {
        self.confirm_empty_trash = false;
        let workspace = self.library.read(cx).workspace_id();
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.empty_trash(workspace).map(drop));
        });
    }

    /// The detail panel's note for a bookmark in Trash.
    pub(super) fn detail_trash(
        &self,
        bookmark: &Bookmark,
        cx: &mut Context<'_, MainView>,
    ) -> Option<Div> {
        let deleted_at = bookmark.deleted_at?;
        let theme = Theme::get(cx);
        Some(
            div()
                .p_3()
                .flex()
                .items_center()
                .gap_2()
                .rounded(CARD_RADIUS)
                .bg(theme.soft)
                .text_sm()
                .child(icon("trash", px(14.), theme.muted))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .child(format!("In Trash. {}", days_left(deleted_at, now()))),
                ),
        )
    }

    /// Restore and Delete now, in place of the usual actions.
    pub(super) fn trash_actions(&self, id: BookmarkId, cx: &mut Context<'_, MainView>) -> Div {
        let theme = Theme::get(cx).clone();
        let library = self.library.clone();
        let restore = library.clone();
        div()
            .flex_none()
            .p_3()
            .flex()
            .gap_2()
            .border_t_1()
            .border_color(theme.hairline)
            .child(
                button("restore", "Restore", Tone::Primary, cx)
                    .flex_1()
                    .on_click(move |_, _, cx| {
                        restore.update(cx, |library, cx| {
                            library.change(cx, move |store| store.restore(id));
                        });
                    }),
            )
            .child(
                button("delete-for-good", "Delete now", Tone::Soft, cx).on_click(
                    move |_, _, cx| {
                        library.update(cx, |library, cx| {
                            library.change(cx, move |store| store.delete_for_good(id));
                        });
                    },
                ),
            )
    }
}

fn days_left(deleted_at: i64, now: i64) -> String {
    const DAY: i64 = 24 * 60 * 60;
    let left = deleted_at + TRASH_SECONDS - now;
    match (left + DAY - 1) / DAY {
        ..=1 => "Deleted for good within a day.".to_owned(),
        days => format!("Deleted for good in {days} days."),
    }
}

fn now() -> i64 {
    jiff::Timestamp::now().as_second()
}
