//! The folder editor's glyph grid: the folder shape and every glyph, all in the folder's
//! color.

use gpui::{
    Context, Div, InteractiveElement as _, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, div, prelude::FluentBuilder as _, px,
};
use knoa_core::{FolderGlyph, FolderIcon};

use super::FolderEditor;
use crate::{
    i18n::{LOADER, t},
    ui::{
        components::{Finish, folder_mark, tip},
        theme::{ROW_RADIUS, Theme},
    },
};

const CELL: f32 = 30.;
const GLYPH: f32 = 18.;

impl FolderEditor {
    pub(super) fn icon_picker(&self, theme: &Theme, cx: &mut Context<'_, FolderEditor>) -> Div {
        let glyphs = FolderGlyph::ALL.iter().map(|&glyph| Some(glyph.into()));
        let choices = std::iter::once(None).chain(glyphs);
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
            None => t!("main-folder-icon-none"),
            Some(FolderIcon::Glyph(glyph)) => glyph_label(*glyph),
            Some(FolderIcon::Emoji(_)) => t!("main-folder-icon-emoji"),
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
            .on_click(cx.listener(move |this, _, window, cx| {
                this.icon.clone_from(&icon);
                this.clear_typed_emoji(window, cx);
                cx.notify();
            }))
    }
}

/// What a glyph stands for, in the current language: "Reading", "Travel". Its message ID
/// is built at run time, so a test checks that each one exists.
fn glyph_label(glyph: FolderGlyph) -> String {
    LOADER.get(&glyph_id(glyph))
}

fn glyph_id(glyph: FolderGlyph) -> String {
    format!("main-glyph-{}", glyph.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_has_a_message() {
        for &glyph in FolderGlyph::ALL {
            let id = glyph_id(glyph);
            assert!(LOADER.has(&id), "{id} is missing");
        }
    }
}
