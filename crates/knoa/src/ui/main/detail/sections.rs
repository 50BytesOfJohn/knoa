//! The detail panel's read-only parts: summary, status notice, facts and provenance.

use gpui::{
    Context, Div, FontWeight, HighlightStyle, Hsla, IntoElement as _, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, StyledText, div, prelude::FluentBuilder as _, px,
};
use jiff::{Timestamp, tz::TimeZone};
use knoa_core::{Bookmark, Status};

use super::MainView;
use crate::{
    services::organizer::Organizer,
    ui::{
        components::{Tone, button, icon},
        mascot::{Mood, ai_mark, mascot},
        theme::Theme,
    },
};

/// The AI's dot as a character, for the end of a line of text it wrote.
const AI_MARK: &str = "●";

/// The summary, ending in the AI's dot when the AI wrote it.
pub(super) fn summary(bookmark: &Bookmark, theme: &Theme) -> Option<Div> {
    let summary = bookmark.summary.as_ref()?;
    let text = div()
        .mt_1()
        .text_sm()
        .line_height(px(21.))
        .text_color(theme.text);
    if !bookmark.status.is_organized() {
        return Some(text.child(summary.clone()));
    }
    let marked = format!("{summary} {AI_MARK}");
    let mark = marked.len() - AI_MARK.len()..marked.len();
    let style = HighlightStyle {
        color: Some(theme.ai),
        ..HighlightStyle::default()
    };
    Some(text.child(StyledText::new(marked).with_highlights([(mark, style)])))
}

impl MainView {
    /// The one thing about this bookmark that needs attention, if any: it is in Trash, the
    /// workspace's guideline set it aside, or it isn't organized yet.
    pub(super) fn detail_notice(
        &self,
        bookmark: &Bookmark,
        cx: &mut Context<'_, MainView>,
    ) -> Option<Div> {
        if bookmark.deleted_at.is_some() {
            return self.detail_trash(bookmark, cx);
        }
        if let Some(reason) = bookmark.set_aside.clone() {
            return Some(self.set_aside_notice(bookmark, reason, cx));
        }
        self.organize_notice(bookmark, cx)
    }

    fn set_aside_notice(
        &self,
        bookmark: &Bookmark,
        reason: String,
        cx: &mut Context<'_, MainView>,
    ) -> Div {
        let theme = Theme::get(cx).clone();
        let (id, library) = (bookmark.id, self.library.clone());
        let text = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_0p5()
            .child(
                div()
                    .font_weight(FontWeight::MEDIUM)
                    .child("Set aside: doesn't fit this workspace"),
            )
            .child(div().text_color(theme.muted).child(reason));
        notice(theme.soft)
            .child(text)
            .child(
                button("keep", "Keep", Tone::Primary, cx).on_click(move |_, _, cx| {
                    library.update(cx, |library, cx| {
                        library.change(cx, move |store| store.keep(id));
                    });
                }),
            )
    }

    fn organize_notice(&self, bookmark: &Bookmark, cx: &mut Context<'_, MainView>) -> Option<Div> {
        let (message, action) = match bookmark.status {
            status if status.is_organized() => return None,
            Status::Queued | Status::Organizing => ("Organizing this one now…", None),
            Status::Failed => ("Organizing didn't work out.", Some("Try again")),
            _ if bookmark.summary.is_none() => {
                ("Only the title and link so far.", Some("Organize"))
            }
            _ => ("Read from the page only, without AI.", Some("Organize")),
        };
        let theme = Theme::get(cx).clone();
        let (id, library) = (bookmark.id, self.library.clone());
        let organizing = matches!(bookmark.status, Status::Queued | Status::Organizing);
        Some(
            notice(theme.ai_soft)
                .when(organizing, |s| {
                    s.child(mascot(
                        "organize-notice",
                        px(24.),
                        Mood::Organizing,
                        theme.ai,
                        theme.ai_soft,
                    ))
                })
                .child(div().flex_1().min_w_0().child(message))
                .when_some(action, |s, label| {
                    s.child(
                        button("organize", label, Tone::Soft, cx).on_click(move |_, _, cx| {
                            library.update(cx, |library, cx| {
                                library.change(cx, move |store| store.queue(id));
                            });
                            Organizer::wake(cx);
                        }),
                    )
                }),
        )
    }
}

/// A rounded, tinted strip for a status message and its action.
fn notice(bg: Hsla) -> Div {
    div()
        .p_3()
        .flex()
        .items_center()
        .gap_3()
        .rounded(px(12.))
        .bg(bg)
        .text_sm()
}

/// A quiet label over a group of values.
pub(super) fn section(title: &str, theme: &Theme) -> Div {
    div().flex().flex_col().gap_2p5().child(
        div()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.faint)
            .child(title.to_owned()),
    )
}

/// The kind's typed fields, then the extras, as a label and value grid.
pub(super) fn fields(bookmark: &Bookmark, theme: &Theme) -> Option<Div> {
    let kind = bookmark.kind?;
    let mut rows: Vec<(String, String)> = kind
        .definition()
        .fields
        .iter()
        .filter_map(|def| Some((def.label(), def.display(bookmark.fields.get(&def.name)?))))
        .filter(|(_, v)| !v.is_empty())
        .collect();
    rows.extend(
        bookmark
            .extras
            .iter()
            .map(|(name, value)| (capitalize(name), value.clone())),
    );
    if rows.is_empty() {
        return None;
    }
    let mut grid = div().flex().flex_col().gap_2();
    for (label, value) in rows {
        grid = grid.child(
            div()
                .flex()
                .gap_3()
                .text_sm()
                .line_height(px(19.))
                .child(div().w(px(92.)).flex_none().text_color(theme.muted).child(label))
                // `min_w_0` lets long values wrap instead of running past the panel's edge.
                .child(div().flex_1().min_w_0().text_color(theme.text).child(value)),
        );
    }
    Some(section("Details", theme).child(grid))
}

/// Where the bookmark came from, when it was saved and which model organized it: the
/// "why" behind the values marked with the AI's dot.
pub(super) fn provenance(bookmark: &Bookmark, theme: &Theme) -> Option<Div> {
    let glyph = |name: &str| icon(name, px(12.), theme.faint).into_any_element();
    let mut lines = Vec::new();
    if let Some(folder) = &bookmark.browser_folder {
        lines.push((glyph("folder"), folder.replace('/', " › ")));
    }
    if bookmark.removed_from_browser {
        lines.push((
            glyph("unlink"),
            "Removed from the browser, kept here".to_owned(),
        ));
    }
    if let Some(saved) = saved_on(bookmark.added_at) {
        lines.push((glyph("calendar"), saved));
    }
    if let Some(model) = &bookmark.model {
        let name = model.rsplit('/').next().unwrap_or(model);
        lines.push((
            ai_mark(px(12.), theme.ai).into_any_element(),
            format!("Marks what {name} chose"),
        ));
    }
    if lines.is_empty() {
        return None;
    }
    let mut column = div()
        .pt_4()
        .border_t_1()
        .border_color(theme.hairline)
        .flex()
        .flex_col()
        .gap_1p5()
        .text_xs()
        .text_color(theme.faint);
    for (leading, text) in lines {
        column = column.child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(leading)
                .child(div().min_w_0().truncate().child(text)),
        );
    }
    Some(column)
}

fn saved_on(added_at: i64) -> Option<String> {
    let tz = TimeZone::system();
    let added = Timestamp::from_second(added_at).ok()?.to_zoned(tz.clone());
    let this_year = Timestamp::now().to_zoned(tz).year();
    let format = if added.year() == this_year {
        "%-d %B"
    } else {
        "%-d %B %Y"
    };
    Some(format!("Saved {}", added.strftime(format)))
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}
