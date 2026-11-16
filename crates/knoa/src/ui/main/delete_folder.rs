//! Asking before a folder is deleted, saying how many subfolders go with it and that its
//! bookmarks stay.

use gpui::{
    Context, FontWeight, IntoElement, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, Window, div, px,
};
use knoa_core::FolderId;

use super::{MainView, overlay::Overlay};
use crate::ui::{
    components::{Tone, button},
    theme::{SERIF, Theme},
};

impl MainView {
    pub fn confirm_delete_folder(
        &mut self,
        id: FolderId,
        _: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.open_overlay(Overlay::DeleteFolder(id), cx);
    }

    fn delete_folder(&mut self, id: FolderId, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| store.delete_folder(id));
        });
        self.close_overlay(window, cx);
    }

    pub(super) fn delete_folder_prompt(
        &self,
        id: FolderId,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let tree = self.library.read(cx).folder_tree();
        let name = tree.get(id).map(|f| f.name.clone()).unwrap_or_default();
        let detail = match tree.descendants(id).len() {
            0 => "Its bookmarks stay in your library.".to_owned(),
            1 => {
                "Its subfolder is deleted with it. Their bookmarks stay in your library.".to_owned()
            }
            n => format!(
                "Its {n} subfolders are deleted with it. Their bookmarks stay in your library."
            ),
        };
        div()
            .w(px(360.))
            .p_5()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .font_family(SERIF)
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(format!("Delete “{name}”?")),
            )
            .child(div().text_sm().text_color(theme.muted).child(detail))
            .child(
                div()
                    .mt_2()
                    .flex()
                    .gap_2()
                    .child(div().flex_1())
                    .child(
                        button("cancel-delete-folder", "Cancel", Tone::Ghost, cx).on_click(
                            cx.listener(|this, _, window, cx| this.close_overlay(window, cx)),
                        ),
                    )
                    .child(
                        button("delete-folder", "Delete", Tone::Primary, cx).on_click(cx.listener(
                            move |this, _, window, cx| this.delete_folder(id, window, cx),
                        )),
                    ),
            )
    }
}
