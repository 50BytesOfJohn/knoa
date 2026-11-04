//! The floating detail panel for the selected bookmark.

use gpui::{
    ClipboardItem, Context, Div, FontWeight, InteractiveElement as _, IntoElement, ObjectFit,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _, StyledImage as _, Window,
    div, img, prelude::FluentBuilder as _, px,
};
use knoa_core::{Bookmark, Origin, Status};

use super::MainView;
use crate::{
    services::organizer::Organizer,
    ui::{
        components::{Tone, button, chip, icon, kind_icon, site_mark},
        theme::{DETAIL_WIDTH, MARGIN, PANEL_RADIUS, SERIF, Theme},
    },
};

impl MainView {
    pub(super) fn render_detail(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let Some(bookmark) = self
            .selected
            .and_then(|id| self.library.read(cx).bookmark(id).cloned())
        else {
            return div().into_any_element();
        };
        let body = div()
            .id("detail-body")
            .flex_1()
            .overflow_y_scroll()
            .child(self.detail_header(&bookmark, cx))
            .child(
                div()
                    .px_5()
                    .pb_5()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .children(self.detail_set_aside(&bookmark, cx))
                    .child(self.detail_about(&bookmark, cx))
                    .children(fields(&bookmark, &theme))
                    .children(self.detail_tags(&bookmark, cx))
                    .child(self.detail_folders(&bookmark, cx))
                    .children(origin(&bookmark, &theme)),
            );
        div()
            .absolute()
            .top(MARGIN)
            .right(MARGIN)
            .bottom(MARGIN)
            .w(DETAIL_WIDTH)
            .flex()
            .flex_col()
            .overflow_hidden()
            .rounded(PANEL_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_lg()
            .child(body)
            .child(self.detail_actions(&bookmark, cx))
            .into_any_element()
    }

    fn detail_header(&self, bookmark: &Bookmark, cx: &mut Context<'_, MainView>) -> Div {
        let theme = Theme::get(cx);
        let close = div()
            .id("close-detail")
            .absolute()
            .top_3()
            .right_3()
            .size(px(28.))
            .rounded_full()
            .bg(theme.panel.opacity(0.85))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .child(icon("x", px(15.), theme.muted))
            .on_click(cx.listener(|this, _, _, cx| {
                this.detail_open = false;
                cx.notify();
            }));
        let mut header = div().relative().w_full().child(close);
        header = match &bookmark.image {
            Some(image) => {
                let height =
                    (f32::from(DETAIL_WIDTH) * image.aspect_ratio().clamp(0.4, 1.3)).min(380.);
                header.child(
                    div().h(px(height)).w_full().bg(theme.soft).child(
                        img(image.path.clone())
                            .size_full()
                            .object_fit(ObjectFit::Cover),
                    ),
                )
            }
            None => header.child(div().h(px(48.))),
        };
        header.child(
            div()
                .px_5()
                .pt_4()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .text_xs()
                        .text_color(theme.muted)
                        .child(icon(kind_icon(bookmark.kind), px(13.), theme.muted))
                        .child(bookmark.kind.map_or("Bookmark", |k| k.label()).to_owned())
                        .child("·")
                        .child(site_mark(&bookmark.site, px(14.), cx))
                        .child(bookmark.site.clone()),
                )
                .child(
                    div()
                        .font_family(SERIF)
                        .text_size(px(24.))
                        .line_height(px(30.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(bookmark.title.clone()),
                ),
        )
    }

    fn detail_about(&self, bookmark: &Bookmark, cx: &mut Context<'_, MainView>) -> Div {
        let theme = Theme::get(cx).clone();
        let organized = bookmark.status.is_organized();
        let mut about = div().flex().flex_col().gap_2();
        if let Some(summary) = &bookmark.summary {
            about = about.child(
                div()
                    .flex()
                    .gap_2()
                    .text_sm()
                    .line_height(px(20.))
                    .when(organized, |s| {
                        s.child(div().text_color(theme.ai).child("✦"))
                    })
                    .child(div().flex_1().min_w_0().child(summary.clone())),
            );
        }
        if organized {
            return about;
        }
        let (message, action) = match bookmark.status {
            Status::Queued | Status::Organizing => ("✦ Organizing this one now…", None),
            Status::Failed => (
                "Organizing didn't work out. Want to try again?",
                Some("Try again"),
            ),
            _ if bookmark.summary.is_none() => {
                ("Only the title and link so far.", Some("Organize"))
            }
            _ => (
                "Organized from the page only, without AI.",
                Some("Organize"),
            ),
        };
        let id = bookmark.id;
        about.child(
            div()
                .p_3()
                .flex()
                .items_center()
                .gap_3()
                .rounded(px(12.))
                .bg(theme.ai_soft)
                .text_sm()
                .child(div().flex_1().text_color(theme.text).child(message))
                .when_some(action, |s, label| {
                    let library = self.library.clone();
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

    /// Why the workspace's guideline set this bookmark aside, and the way to keep it.
    fn detail_set_aside(&self, bookmark: &Bookmark, cx: &mut Context<'_, MainView>) -> Option<Div> {
        let reason = bookmark.set_aside.clone()?;
        let theme = Theme::get(cx).clone();
        let (id, library) = (bookmark.id, self.library.clone());
        Some(
            div()
                .p_3()
                .flex()
                .items_center()
                .gap_3()
                .rounded(px(12.))
                .bg(theme.soft)
                .text_sm()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .child("✦ Set aside: doesn't fit this workspace"),
                        )
                        .child(div().text_color(theme.muted).child(reason)),
                )
                .child(
                    button("keep", "Keep", Tone::Primary, cx).on_click(move |_, _, cx| {
                        library.update(cx, |library, cx| {
                            library.change(cx, move |store| store.keep(id));
                        });
                    }),
                ),
        )
    }

    fn detail_tags(&self, bookmark: &Bookmark, cx: &mut Context<'_, MainView>) -> Option<Div> {
        if bookmark.tags.is_empty() {
            return None;
        }
        let theme = Theme::get(cx).clone();
        let mut row = div().flex().flex_wrap().gap_1p5();
        for (index, (tag, origin)) in bookmark.tags.iter().enumerate() {
            let (id, tag_name) = (bookmark.id, tag.clone());
            let library = self.library.clone();
            row = row.child(
                div()
                    .id(("tag", index))
                    .group("tag")
                    .flex()
                    .items_center()
                    .gap_1()
                    .h(px(24.))
                    .px_2()
                    .rounded_full()
                    .bg(theme.soft)
                    .text_xs()
                    .when(*origin == Origin::Ai, |s| {
                        s.child(div().text_color(theme.ai).child("✦"))
                    })
                    .child(format!("#{tag}"))
                    .child(
                        div()
                            .id(("remove-tag", index))
                            .cursor_pointer()
                            .child(icon("x", px(11.), theme.faint))
                            .on_click(move |_, _, cx| {
                                let tag = tag_name.clone();
                                library.update(cx, |library, cx| {
                                    library.change(cx, move |store| store.remove_tag(id, &tag));
                                });
                            }),
                    ),
            );
        }
        Some(section("Tags", &theme).child(row))
    }

    fn detail_folders(&self, bookmark: &Bookmark, cx: &mut Context<'_, MainView>) -> Div {
        let theme = Theme::get(cx).clone();
        let folders = self.library.read(cx).folders().to_vec();
        let mut row = div().flex().flex_wrap().gap_1p5();
        for folder in &folders {
            let link = bookmark.folders.iter().find(|l| l.folder == folder.id);
            let color = theme.folder_color(folder.color.0);
            let (id, folder_id, library) = (bookmark.id, folder.id, self.library.clone());
            let member = link.is_some();
            let mut item = div()
                .id(("folder-chip", folder.id.0 as usize))
                .flex()
                .items_center()
                .gap_1()
                .h(px(26.))
                .px_2p5()
                .rounded_full()
                .text_xs()
                .cursor_pointer()
                .map(|s| {
                    if member {
                        s.bg(color.opacity(0.18)).text_color(theme.text)
                    } else {
                        s.border_1()
                            .border_color(theme.hairline)
                            .text_color(theme.muted)
                            .hover(|s| s.bg(theme.soft))
                    }
                })
                .child(div().size(px(7.)).rounded_full().bg(color))
                .child(folder.name.clone());
            if let Some(link) = link.filter(|l| l.origin == Origin::Ai) {
                item = item.child(div().text_color(theme.ai).child("✦"));
                if let Some(p) = link.probability.filter(|p| *p < 1.) {
                    item = item.child(
                        div()
                            .text_color(theme.faint)
                            .child(format!("{:.0}%", p * 100.)),
                    );
                }
                item = item.child(div().ml_1().text_color(theme.muted).child("Undo"));
            }
            row = row.child(item.on_click(move |_, _, cx| {
                library.update(cx, |library, cx| {
                    library.change(cx, move |store| {
                        if member {
                            store.remove_from_folder(id, folder_id)
                        } else {
                            store.add_to_folder(id, folder_id)
                        }
                    });
                });
            }));
        }
        if folders.is_empty() {
            row = row.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("No folders yet. Make one in the sidebar."),
            );
        }
        section("Folders", &theme).child(row)
    }

    fn detail_actions(&self, bookmark: &Bookmark, cx: &mut Context<'_, MainView>) -> Div {
        let theme = Theme::get(cx).clone();
        let (url, id, up_next) = (bookmark.url.clone(), bookmark.id, bookmark.up_next);
        let copy_url = url.clone();
        div()
            .flex_none()
            .p_3()
            .flex()
            .gap_2()
            .border_t_1()
            .border_color(theme.hairline)
            .child(
                button("open", "Open  ↵", Tone::Primary, cx)
                    .flex_1()
                    .on_click(cx.listener(move |this, _, _, cx| this.open_link(&url, cx))),
            )
            .child(
                button(
                    "up-next",
                    if up_next { "In Up next" } else { "Up next" },
                    Tone::Soft,
                    cx,
                )
                .on_click(cx.listener(move |this, _, _, cx| this.set_up_next(id, cx))),
            )
            .child(
                button("copy", "Copy link", Tone::Ghost, cx).on_click(move |_, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(copy_url.clone()));
                }),
            )
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn section(title: &str, theme: &Theme) -> Div {
    div().flex().flex_col().gap_2().child(
        div()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.faint)
            .child(title.to_owned()),
    )
}

fn fields(bookmark: &Bookmark, theme: &Theme) -> Option<Div> {
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
    let mut grid = div().flex().flex_col().gap_1p5();
    for (label, value) in rows {
        grid = grid.child(
            div()
                .flex()
                .gap_3()
                .text_sm()
                .child(
                    div()
                        .w(px(100.))
                        .flex_none()
                        .text_color(theme.muted)
                        .child(label),
                )
                .child(div().flex_1().child(value)),
        );
    }
    Some(section("Details", theme).child(grid))
}

/// Where the bookmark came from and who organized it: the "why" behind AI values.
fn origin(bookmark: &Bookmark, theme: &Theme) -> Option<Div> {
    let mut notes = Vec::new();
    if let Some(folder) = &bookmark.browser_folder {
        notes.push(format!("From the browser folder {folder}"));
    }
    if bookmark.removed_from_browser {
        notes.push("Removed from the browser; kept here".into());
    }
    if let Some(model) = &bookmark.model {
        notes.push(format!("✦ Organized by {model}"));
    }
    if notes.is_empty() {
        return None;
    }
    let mut column = div()
        .flex()
        .flex_col()
        .gap_1()
        .text_xs()
        .text_color(theme.faint);
    for note in notes {
        column = column.child(note);
    }
    Some(column.child(chip(
        "Values marked ✦ were set by the AI",
        theme.soft,
        theme.muted,
    )))
}
