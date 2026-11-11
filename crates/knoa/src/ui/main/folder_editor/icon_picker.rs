//! The folder editor's icon grid: the folder shape, then every glyph.

use gpui::{
    Context, Div, InteractiveElement as _, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, div, prelude::FluentBuilder as _, px,
};
use knoa_core::FolderIcon;

use super::FolderEditor;
use crate::ui::{
    components::{Finish, folder_mark, tip},
    theme::{ROW_RADIUS, Theme},
};

const CELL: f32 = 30.;
const GLYPH: f32 = 18.;

impl FolderEditor {
    pub(super) fn icon_picker(&self, theme: &Theme, cx: &mut Context<'_, FolderEditor>) -> Div {
        let choices = std::iter::once(None).chain(FolderIcon::ALL.iter().copied().map(Some));
        let mut grid = div().flex().flex_wrap().gap_1();
        for (index, glyph) in choices.enumerate() {
            grid = grid.child(self.icon_cell(index, glyph, theme, cx));
        }
        grid
    }

    fn icon_cell(
        &self,
        index: usize,
        glyph: Option<FolderIcon>,
        theme: &Theme,
        cx: &mut Context<'_, FolderEditor>,
    ) -> impl gpui::IntoElement {
        let selected = glyph == self.icon;
        let color = theme.folder_color(self.color);
        let label = glyph.map_or("Folder", FolderIcon::label);
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
                glyph,
                if selected { color } else { theme.muted },
                px(GLYPH),
                Finish::Line,
            ))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.icon = glyph;
                cx.notify();
            }))
    }
}
