//! Tags and folders in the detail panel: where a bookmark is filed, and undoing the AI's
//! choices in one click.

use gpui::{
    Context, Div, Hsla, InteractiveElement as _, ParentElement as _, Stateful,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use knoa_core::{Bookmark, Folder, FolderLink, Origin};

use super::{MainView, sections::section};
use crate::ui::{components::icon, theme::Theme};

/// Below this, the AI's confidence in a folder is worth showing; above it, it's noise.
const UNSURE: f32 = 0.8;

impl MainView {
    pub(super) fn detail_tags(
        &self,
        bookmark: &Bookmark,
        cx: &mut Context<'_, MainView>,
    ) -> Option<Div> {
        if bookmark.tags.is_empty() {
            return None;
        }
        let theme = Theme::get(cx).clone();
        let mut row = div().flex().flex_wrap().gap_1p5();
        for (index, (tag, origin)) in bookmark.tags.iter().enumerate() {
            let (id, tag_name, library) = (bookmark.id, tag.clone(), self.library.clone());
            let item = pill(("tag", index))
                .bg(theme.soft)
                .text_color(theme.text)
                .child(removable(
                    if *origin == Origin::Ai {
                        ai_mark(&theme)
                    } else {
                        div().text_color(theme.faint).child("#")
                    },
                    &theme,
                ))
                .child(tag.clone())
                .tooltip(super::tip("Remove tag", None, &theme))
                .on_click(move |_, _, cx| {
                    let tag = tag_name.clone();
                    library.update(cx, |library, cx| {
                        library.change(cx, move |store| store.remove_tag(id, &tag));
                    });
                });
            row = row.child(item);
        }
        Some(section("Tags", &theme).child(row))
    }

    /// The bookmark's folders, then a way to add it to another. With no folders yet, every
    /// folder is offered straight away.
    pub(super) fn detail_folders(
        &self,
        bookmark: &Bookmark,
        cx: &mut Context<'_, MainView>,
    ) -> Div {
        let theme = Theme::get(cx).clone();
        let folders = self.library.read(cx).folders().to_vec();
        if folders.is_empty() {
            return section("Folders", &theme).child(
                div()
                    .text_sm()
                    .text_color(theme.muted)
                    .child("No folders yet. Make one in the sidebar."),
            );
        }
        let link_of = |folder: &Folder| bookmark.folders.iter().find(|l| l.folder == folder.id);
        let (filed, unfiled): (Vec<&Folder>, Vec<&Folder>) =
            folders.iter().partition(|f| link_of(f).is_some());
        let picking = filed.is_empty() || self.folder_picker == Some(bookmark.id);
        let mut row = div().flex().flex_wrap().gap_1p5();
        for folder in &filed {
            let link = link_of(folder).copied();
            row = row.child(self.filed_chip(bookmark, folder, link, &theme));
        }
        if picking {
            for folder in &unfiled {
                row = row.child(self.unfiled_chip(bookmark, folder, &theme));
            }
        } else if !unfiled.is_empty() {
            row = row.child(self.more_folders_chip(bookmark, &theme, cx));
        }
        section("Folders", &theme).child(row)
    }

    /// A folder the bookmark is in. Clicking takes it out, which is also how an AI choice
    /// is undone.
    fn filed_chip(
        &self,
        bookmark: &Bookmark,
        folder: &Folder,
        link: Option<FolderLink>,
        theme: &Theme,
    ) -> Stateful<Div> {
        let color = theme.folder_color(folder.color.0);
        let (id, folder_id, library) = (bookmark.id, folder.id, self.library.clone());
        let by_ai = link.as_ref().is_some_and(|l| l.origin == Origin::Ai);
        let unsure = link
            .and_then(|l| l.probability)
            .filter(|p| by_ai && *p < UNSURE);
        let tip = if by_ai {
            "Undo: take it out"
        } else {
            "Take it out"
        };
        pill(("folder", folder.id.0 as usize))
            .bg(color.opacity(0.16))
            .text_color(theme.text)
            .child(removable(swatch(color), theme))
            .child(folder.name.clone())
            .when(by_ai, |s| s.child(ai_mark(theme)))
            .when_some(unsure, |s, p| {
                s.child(
                    div()
                        .text_color(theme.muted)
                        .child(format!("{:.0}%", p * 100.)),
                )
            })
            .tooltip(super::tip(tip, None, theme))
            .on_click(move |_, _, cx| {
                library.update(cx, |library, cx| {
                    library.change(cx, move |store| store.remove_from_folder(id, folder_id));
                });
            })
    }

    fn unfiled_chip(&self, bookmark: &Bookmark, folder: &Folder, theme: &Theme) -> Stateful<Div> {
        let color = theme.folder_color(folder.color.0);
        let (id, folder_id, library) = (bookmark.id, folder.id, self.library.clone());
        let hover = theme.soft;
        pill(("folder", folder.id.0 as usize))
            .border_1()
            .border_color(theme.hairline)
            .text_color(theme.muted)
            .hover(move |s| s.bg(hover))
            .child(icon("plus", px(12.), color))
            .child(folder.name.clone())
            .on_click(move |_, _, cx| {
                library.update(cx, |library, cx| {
                    library.change(cx, move |store| store.add_to_folder(id, folder_id));
                });
            })
    }

    fn more_folders_chip(
        &self,
        bookmark: &Bookmark,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> Stateful<Div> {
        let id = bookmark.id;
        let hover = theme.soft;
        pill("more-folders")
            .px_2()
            .border_1()
            .border_color(theme.hairline)
            .text_color(theme.muted)
            .hover(move |s| s.bg(hover))
            .child(icon("plus", px(12.), theme.muted))
            .tooltip(super::tip("Add to another folder", None, theme))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.folder_picker = Some(id);
                cx.notify();
            }))
    }
}

fn pill(id: impl Into<gpui::ElementId>) -> Stateful<Div> {
    div()
        .id(id)
        .group("pill")
        .flex()
        .items_center()
        .gap_1p5()
        .h(px(26.))
        .px_2p5()
        .rounded_full()
        .text_xs()
        .cursor_pointer()
}

fn swatch(color: Hsla) -> Div {
    div().size(px(7.)).flex_none().rounded_full().bg(color)
}

fn ai_mark(theme: &Theme) -> Div {
    div().text_color(theme.ai).child("✦")
}

/// A chip's leading mark, which turns into an × on hover: the chips stay calm until you
/// reach for one, and the × takes no room of its own.
fn removable(mark: Div, theme: &Theme) -> Div {
    let center = || {
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
    };
    div()
        .relative()
        .size(px(12.))
        .flex_none()
        .child(
            center()
                .group_hover("pill", gpui::Styled::invisible)
                .child(mark),
        )
        .child(
            center()
                .invisible()
                .group_hover("pill", gpui::Styled::visible)
                .child(icon("x", px(11.), theme.text)),
        )
}
