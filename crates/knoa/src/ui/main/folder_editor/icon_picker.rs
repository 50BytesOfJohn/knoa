//! The folder editor's icon grid: the folder shape, every glyph, and the folder's emoji
//! when it has one.

use gpui::{
    Context, Div, InteractiveElement as _, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, div, prelude::FluentBuilder as _, px,
};
use knoa_core::{FolderGlyph, FolderIcon};

use super::FolderEditor;
use crate::ui::{
    components::{Finish, folder_mark, tip},
    theme::{ROW_RADIUS, Theme},
};

const CELL: f32 = 30.;
const GLYPH: f32 = 18.;

impl FolderEditor {
    pub(super) fn icon_picker(&self, theme: &Theme, cx: &mut Context<'_, FolderEditor>) -> Div {
        let glyphs = FolderGlyph::ALL.iter().map(|&glyph| Some(glyph.into()));
        let emoji = self
            .icon
            .clone()
            .filter(|icon| matches!(icon, FolderIcon::Emoji(_)));
        let choices = std::iter::once(None).chain(glyphs).chain(emoji.map(Some));
        let mut grid = div().flex().flex_wrap().gap_1();
        for (index, glyph) in choices.enumerate() {
            grid = grid.child(self.icon_cell(index, glyph, theme, cx));
        }
        grid
    }

    fn icon_cell(
        &self,
        index: usize,
        icon: Option<FolderIcon>,
        theme: &Theme,
        cx: &mut Context<'_, FolderEditor>,
    ) -> impl gpui::IntoElement {
        let selected = icon == self.icon;
        let color = theme.folder_color(self.color);
        let label = match &icon {
            None => "Folder",
            Some(FolderIcon::Glyph(glyph)) => glyph.label(),
            Some(FolderIcon::Emoji(_)) => "Emoji",
        };
        let hover = theme.hover;
        div()
            .id(("folder-icon", index))
            .size(px(CELL))
            .flex()
            .items_center()
            .justify_center()
            .rounded(ROW_RADIUS)
            .cursor_pointer()
            .when(selected, |s| s.bg(color.opacity(0.16)))
            .when(!selected, |s| s.hover(move |s| s.bg(hover)))
            .tooltip(tip(label, None, theme))
            .child(folder_mark(
                icon.as_ref(),
                if selected { color } else { theme.muted },
                px(GLYPH),
                Finish::Line,
            ))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.icon.clone_from(&icon);
                cx.notify();
            }))
    }
}
