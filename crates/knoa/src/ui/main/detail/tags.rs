//! Tags in the detail panel: adding them, with the workspace's own tags as suggestions,
//! and removing them in a way that can be taken back.

use gpui::{
    Context, Div, InteractiveElement as _, ParentElement as _, Stateful,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    Sizable as _,
    input::{Escape, Input},
};
use knoa_core::{Bookmark, BookmarkId, Origin, normalize_tag};

mod edits;

pub(in crate::ui::main) use edits::TagEdits;
use edits::TagField;

use super::{
    MainView,
    filing::{pill, removable},
    sections::section,
};
use crate::ui::{components::icon, mascot::ai_mark, theme::Theme};

/// Enough suggestions to find a tag without the list crowding the panel.
const MAX_SUGGESTIONS: usize = 6;

/// A tag as the panel shows it: on the bookmark, or just taken off.
struct Shown {
    name: String,
    origin: Origin,
    removed: bool,
}

impl MainView {
    pub(super) fn detail_tags(
        &self,
        bookmark: &Bookmark,
        in_trash: bool,
        cx: &mut Context<'_, MainView>,
    ) -> Option<Div> {
        let shown = self.shown_tags(bookmark);
        if shown.is_empty() && in_trash {
            return None;
        }
        let theme = Theme::get(cx).clone();
        let mut row = div().flex().flex_wrap().gap_1p5();
        for (index, tag) in shown.iter().enumerate() {
            row = row.child(if tag.removed {
                self.removed_chip(bookmark.id, tag, index, &theme, cx)
            } else {
                self.tag_chip(bookmark.id, tag, index, &theme, cx)
            });
        }
        let field = self.tag_field(bookmark.id);
        if !in_trash {
            row = row.child(match field {
                Some(field) => self.field_chip(field, &theme, cx),
                None => self.add_tag_chip(bookmark.id, shown.is_empty(), &theme, cx),
            });
        }
        let suggestions = field
            .map(|field| {
                let typed = field.input.read(cx).value();
                suggestions(self.library.read(cx).tags(), &shown, &typed)
            })
            .unwrap_or_default();
        Some(
            section("Tags", &theme)
                .child(row)
                .when(!suggestions.is_empty(), |s| {
                    s.child(self.suggestion_row(bookmark.id, suggestions, &theme, cx))
                }),
        )
    }

    /// The bookmark's tags and the ones just taken off, in one stable order, so a removed
    /// tag stays where it was.
    fn shown_tags(&self, bookmark: &Bookmark) -> Vec<Shown> {
        let removed = self
            .tag_edits
            .as_ref()
            .filter(|edits| edits.bookmark == bookmark.id)
            .map_or(&[][..], |edits| &edits.removed);
        let mut shown: Vec<Shown> = bookmark
            .tags
            .iter()
            .filter(|(name, _)| !removed.iter().any(|(r, _)| r == name))
            .map(|(name, origin)| Shown {
                name: name.clone(),
                origin: *origin,
                removed: false,
            })
            .chain(removed.iter().map(|(name, origin)| Shown {
                name: name.clone(),
                origin: *origin,
                removed: true,
            }))
            .collect();
        shown.sort_by(|a, b| a.name.cmp(&b.name));
        shown
    }

    fn tag_chip(
        &self,
        bookmark: BookmarkId,
        tag: &Shown,
        index: usize,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> Stateful<Div> {
        let (name, origin) = (tag.name.clone(), tag.origin);
        let mark = if origin == Origin::Ai {
            ai_mark(px(12.), theme.ai)
        } else {
            div().text_color(theme.faint).child("#")
        };
        pill(("tag", index))
            .bg(theme.soft)
            .text_color(theme.text)
            .child(removable(mark, theme))
            .child(tag.name.clone())
            .tooltip(super::tip("Remove tag", None, theme))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.remove_tag(bookmark, name.clone(), origin, cx);
            }))
    }

    fn removed_chip(
        &self,
        bookmark: BookmarkId,
        tag: &Shown,
        index: usize,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> Stateful<Div> {
        let (name, origin) = (tag.name.clone(), tag.origin);
        let hover = theme.soft;
        pill(("tag", index))
            .border_1()
            .border_dashed()
            .border_color(theme.hairline)
            .text_color(theme.faint)
            .hover(move |s| s.bg(hover))
            .child(icon("undo-2", px(12.), theme.muted))
            .child(div().line_through().child(tag.name.clone()))
            .tooltip(super::tip("Put back", None, theme))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.put_back_tag(bookmark, name.clone(), origin, cx);
            }))
    }

    /// Spelled out while the bookmark has no tags, a bare + once it has some, like the
    /// folders' chip.
    fn add_tag_chip(
        &self,
        bookmark: BookmarkId,
        spelled_out: bool,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> Stateful<Div> {
        let hover = theme.soft;
        pill("add-tag")
            .when(!spelled_out, gpui::Styled::px_2)
            .border_1()
            .border_color(theme.hairline)
            .text_color(theme.muted)
            .hover(move |s| s.bg(hover))
            .child(icon("plus", px(12.), theme.muted))
            .when(spelled_out, |s| s.child("Add tag"))
            .when(!spelled_out, |s| {
                s.tooltip(super::tip("Add a tag", None, theme))
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.open_tag_field(bookmark, window, cx);
            }))
    }

    fn field_chip(
        &self,
        field: &TagField,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> Stateful<Div> {
        pill("tag-field")
            .cursor_text()
            .pl_2p5()
            .pr_1()
            .border_1()
            .border_color(theme.accent)
            .text_color(theme.text)
            .child(div().text_color(theme.faint).child("#"))
            .child(
                div().w(px(120.)).child(
                    Input::new(&field.input)
                        .appearance(false)
                        .cleanable(false)
                        .xsmall(),
                ),
            )
            .on_action(cx.listener(|this, _: &Escape, window, cx| {
                this.close_tag_field(window, cx);
            }))
    }

    fn suggestion_row(
        &self,
        bookmark: BookmarkId,
        suggestions: Vec<String>,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> Div {
        let hover = theme.soft;
        let mut row = div().flex().flex_wrap().gap_1p5();
        for (index, name) in suggestions.into_iter().enumerate() {
            row = row.child(
                pill(("tag-suggestion", index))
                    .h(px(22.))
                    .px_2()
                    .gap_1()
                    .text_color(theme.muted)
                    .hover(move |s| s.bg(hover))
                    .child(icon("plus", px(11.), theme.faint))
                    .child(name.clone())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.add_tags(bookmark, &name, cx);
                        this.clear_tag_field(window, cx);
                    })),
            );
        }
        row
    }
}

/// The workspace's tags that match what's typed, most used first, with tags that start
/// with it ahead of ones that only contain it.
fn suggestions(vocabulary: &[(String, usize)], shown: &[Shown], typed: &str) -> Vec<String> {
    let typed = normalize_tag(typed);
    let mut matches: Vec<&String> = vocabulary
        .iter()
        .map(|(name, _)| name)
        .filter(|name| !shown.iter().any(|s| &s.name == *name) && name.contains(&typed))
        .collect();
    // A stable sort, so each group stays most used first.
    matches.sort_by_key(|name| !name.starts_with(&typed));
    matches.into_iter().take(MAX_SUGGESTIONS).cloned().collect()
}
