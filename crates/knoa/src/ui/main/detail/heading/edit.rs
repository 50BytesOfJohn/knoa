//! Editing the title and description: both fields open together, Return or leaving them
//! saves what changed, Escape puts everything back.

use gpui::{AppContext as _, Context, Entity, Focusable as _, Subscription, Window};
use gpui_component::input::{InputEvent, TextareaState};
use knoa_core::{Bookmark, BookmarkId};

use crate::{i18n::t, ui::main::MainView};

/// Titles wrap over a few lines in the panel; longer ones scroll in the field.
const TITLE_ROWS: usize = 4;
const SUMMARY_ROWS: (usize, usize) = (2, 10);

/// Which field a double-click puts the caret in.
#[derive(Clone, Copy)]
pub(super) enum Part {
    Title,
    Summary,
}

/// The open fields for the bookmark in the detail panel.
pub(in crate::ui::main) struct HeadingEdit {
    bookmark: BookmarkId,
    pub(super) title: Entity<TextareaState>,
    pub(super) summary: Entity<TextareaState>,
    _events: [Subscription; 2],
}

impl HeadingEdit {
    /// The fields keep the placeholders they were given, so a new language sets them again.
    pub(in crate::ui::main) fn translate(&self, window: &mut Window, cx: &mut gpui::App) {
        set_placeholders(&self.title, &self.summary, window, cx);
    }

    pub(in crate::ui::main) fn is_typing(&self, window: &Window, cx: &gpui::App) -> bool {
        [&self.title, &self.summary]
            .iter()
            .any(|field| field.read(cx).focus_handle(cx).is_focused(window))
    }
}

impl MainView {
    pub(super) fn heading_edit_of(&self, bookmark: BookmarkId) -> Option<&HeadingEdit> {
        self.heading_edit
            .as_ref()
            .filter(|edit| edit.bookmark == bookmark)
    }

    /// Opens the fields on `bookmark`, with the caret in `part`. A title is selected whole,
    /// as a rename is on macOS; in a description the caret goes to the end.
    pub(super) fn edit_heading(
        &mut self,
        bookmark: BookmarkId,
        part: Part,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        if self.heading_edit_of(bookmark).is_none() {
            let Some(row) = self.library.read(cx).bookmark(bookmark).cloned() else {
                return;
            };
            self.heading_edit = Some(self.open_fields(&row, window, cx));
            // The fields learn how many lines their text wraps to while being drawn, so
            // they're drawn once more at that height.
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }
        let Some(edit) = &self.heading_edit else {
            return;
        };
        let field = match part {
            Part::Title => &edit.title,
            Part::Summary => &edit.summary,
        };
        field.update(cx, |input, cx| {
            input.focus(window, cx);
            match part {
                Part::Title => input.select_all(window, cx),
                Part::Summary => {
                    let end = input.value().len();
                    input.set_selected_range(end..end, cx);
                }
            }
        });
        cx.notify();
    }

    fn open_fields(
        &mut self,
        bookmark: &Bookmark,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> HeadingEdit {
        let field = |rows: (usize, usize),
                     text: String,
                     window: &mut Window,
                     cx: &mut Context<'_, MainView>| {
            let input = cx.new(|cx| {
                TextareaState::new(window, cx)
                    .auto_grow(rows.0, rows.1)
                    .submit_on_enter(true)
            });
            input.update(cx, |input, cx| input.set_value(text, window, cx));
            input
        };
        let title = field((1, TITLE_ROWS), bookmark.title.clone(), window, cx);
        let summary = field(
            SUMMARY_ROWS,
            bookmark.summary.clone().unwrap_or_default(),
            window,
            cx,
        );
        set_placeholders(&title, &summary, window, cx);
        let events =
            [&title, &summary].map(|input| cx.subscribe_in(input, window, Self::on_heading_event));
        HeadingEdit {
            bookmark: bookmark.id,
            title,
            summary,
            _events: events,
        }
    }

    /// Return saves; Shift-Return is a new line in the description. Focus moving between
    /// the two fields isn't leaving them, so a blur saves only once it has settled
    /// elsewhere. A click on something that takes no focus is caught by the fields' view.
    fn on_heading_event(
        &mut self,
        _: &Entity<TextareaState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        match event {
            InputEvent::PressEnter { shift: false, .. } => self.save_heading_edit(window, cx),
            InputEvent::Blur => cx.defer_in(window, |this, window, cx| {
                let typing = this
                    .heading_edit
                    .as_ref()
                    .is_some_and(|edit| edit.is_typing(window, cx));
                if !typing {
                    this.save_heading_edit(window, cx);
                }
            }),
            _ => {}
        }
    }

    /// Saves what changed. An emptied title keeps the old one: a bookmark always has one.
    pub(in crate::ui::main::detail) fn save_heading_edit(
        &mut self,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let Some(edit) = self.heading_edit.take() else {
            return;
        };
        let had_focus = edit.is_typing(window, cx);
        let title = one_line(&edit.title.read(cx).value());
        let summary = edit.summary.read(cx).value().trim().to_owned();
        let summary = (!summary.is_empty()).then_some(summary);
        if let Some(bookmark) = self.library.read(cx).bookmark(edit.bookmark).cloned() {
            let id = bookmark.id;
            let renamed = (!title.is_empty() && title != bookmark.title).then_some(title);
            let described = (summary != bookmark.summary).then_some(summary);
            self.library.update(cx, |library, cx| {
                library.change(cx, move |store| {
                    if let Some(title) = renamed {
                        store.rename_bookmark(id, &title)?;
                    }
                    match described {
                        Some(summary) => store.describe_bookmark(id, summary.as_deref()),
                        None => Ok(()),
                    }
                });
            });
        }
        self.leave_heading(had_focus, window, cx);
    }

    pub(super) fn cancel_heading_edit(
        &mut self,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        let had_focus = self
            .heading_edit
            .take()
            .is_some_and(|edit| edit.is_typing(window, cx));
        self.leave_heading(had_focus, window, cx);
    }

    /// Return and Escape hand the keys back to the cards; a click elsewhere already put
    /// focus where it belongs.
    fn leave_heading(
        &mut self,
        had_focus: bool,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        if had_focus {
            self.cards_focus.focus(window, cx);
        }
        cx.notify();
    }
}

fn set_placeholders(
    title: &Entity<TextareaState>,
    summary: &Entity<TextareaState>,
    window: &mut Window,
    cx: &mut gpui::App,
) {
    title.update(cx, |input, cx| {
        input.set_placeholder(t!("detail-title"), window, cx);
    });
    summary.update(cx, |input, cx| {
        input.set_placeholder(t!("detail-add-description"), window, cx);
    });
}

/// A title is one line, however it was typed or pasted.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
