//! The detail panel's more button: the bookmark's menu, the same one a card's
//! right-click opens.

use gpui::{Anchor, Context, IntoElement, StatefulInteractiveElement as _};
use gpui_component::menu::DropdownMenu as _;
use knoa_core::BookmarkId;

use super::{MainView, icon_button};
use crate::ui::{
    components::tip,
    main::{bookmark_menu::bookmark_menu, menu_trigger::MenuTrigger},
    theme::Theme,
};

impl MainView {
    pub(super) fn more_button(
        &self,
        id: BookmarkId,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx);
        let view = cx.entity().downgrade();
        let base =
            icon_button("more", "ellipsis", theme.muted, theme).tooltip(tip("More", None, theme));
        MenuTrigger::new(base, theme.soft)
            // The button sits at the panel's bottom right, so the menu opens up and inward.
            .dropdown_menu_with_anchor(Anchor::BottomRight, move |menu, window, cx| {
                bookmark_menu(menu, &view, id, window, cx)
            })
    }
}
