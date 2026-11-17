//! The chip that follows the pointer while a folder or a bookmark is dragged.

use gpui::{
    AppContext as _, Context, FontWeight, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Window, div, px,
};
use knoa_core::FolderIcon;

use super::{drop::DraggedBookmark, outline::FolderRow};
use crate::ui::{
    components::{Finish, folder_mark, icon},
    theme::{ROW_HEIGHT, ROW_RADIUS, Theme},
};

/// What follows the pointer during a drag: the folder's mark and name, or the bookmark's
/// title, on a small chip.
pub(in crate::ui::main) struct DragPreview {
    label: SharedString,
    mark: PreviewMark,
}

enum PreviewMark {
    Folder(Option<FolderIcon>, knoa_core::FolderColor),
    Bookmark,
}

impl DragPreview {
    pub(super) fn folder(row: &FolderRow) -> DragPreview {
        DragPreview {
            label: row.name.clone(),
            mark: PreviewMark::Folder(row.icon.clone(), row.color),
        }
    }

    pub fn bookmark(dragged: &DraggedBookmark) -> DragPreview {
        DragPreview {
            label: dragged.title.clone(),
            mark: PreviewMark::Bookmark,
        }
    }

    /// The preview as an entity, which is what GPUI's `on_drag` asks for.
    pub fn build(self, cx: &mut gpui::App) -> gpui::Entity<DragPreview> {
        cx.new(|_| self)
    }
}

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let mark = match &self.mark {
            PreviewMark::Folder(glyph, color) => folder_mark(
                glyph.as_ref(),
                theme.folder_color(*color),
                px(15.),
                Finish::Line,
            )
            .into_any_element(),
            PreviewMark::Bookmark => icon("link", px(14.), theme.muted).into_any_element(),
        };
        div()
            .flex()
            .items_center()
            .gap_2()
            .h(ROW_HEIGHT)
            .max_w(px(240.))
            .px_2p5()
            .rounded(ROW_RADIUS)
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_md()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.text)
            .child(mark)
            .child(div().truncate().child(self.label.clone()))
    }
}
