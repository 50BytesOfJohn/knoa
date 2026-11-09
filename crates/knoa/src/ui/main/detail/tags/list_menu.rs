//! A tag chip's right-click menu: putting the tag on the workspace's tag list or taking it
//! off, kept out of sight so the panel stays plain.

use gpui::{Context, Div, IntoElement, Stateful};
use gpui_component::menu::{ContextMenuExt as _, PopupMenuItem};
use knoa_core::TagList;

use crate::ui::main::MainView;

/// The chip with its menu. `name` is the tag it shows.
pub(super) fn with_list_menu(
    chip: Stateful<Div>,
    name: String,
    tag_list: &TagList,
    cx: &mut Context<'_, MainView>,
) -> impl IntoElement {
    let listed = tag_list.contains(&name);
    let full = !listed && tag_list.is_full();
    let view = cx.entity().downgrade();
    chip.context_menu(move |menu, _, _| {
        let label = if full {
            "In tag list (list is full)"
        } else {
            "In tag list"
        };
        let (toggle_view, edit_view, name) = (view.clone(), view.clone(), name.clone());
        menu.item(
            PopupMenuItem::new(label)
                .checked(listed)
                .disabled(full)
                .on_click(move |_, _, cx| {
                    let _ = toggle_view.update(cx, |this, cx| {
                        this.set_listed(name.clone(), !listed, cx);
                    });
                }),
        )
        .separator()
        .item(
            PopupMenuItem::new("Edit tag list…").on_click(move |_, window, cx| {
                let _ = edit_view.update(cx, |this, cx| {
                    let workspace = this.library.read(cx).workspace_id();
                    this.edit_workspace(Some(workspace), window, cx);
                });
            }),
        )
    })
}

impl MainView {
    fn set_listed(&mut self, name: String, listed: bool, cx: &mut Context<'_, MainView>) {
        let workspace = self.library.read(cx).workspace_id();
        self.library.update(cx, |library, cx| {
            library.change(cx, move |store| {
                if listed {
                    store.list_tag(workspace, &name)
                } else {
                    store.unlist_tag(workspace, &name)
                }
            });
        });
    }
}
