//! The detail panel's more button: the bookmark's menu, the same one a card's
//! right-click opens.

use gpui::{
    Anchor, App, Context, Div, Hsla, InteractiveElement, Interactivity, IntoElement, RenderOnce,
    Stateful, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
};
use gpui_component::{Selectable, menu::DropdownMenu};
use knoa_core::BookmarkId;

use super::{MainView, icon_button};
use crate::ui::{components::tip, main::bookmark_menu::bookmark_menu, theme::Theme};

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
        MoreButton {
            base,
            open: false,
            open_bg: theme.soft,
        }
        // The button sits at the panel's bottom right, so the menu opens up and inward.
        .dropdown_menu_with_anchor(Anchor::BottomRight, move |menu, window, cx| {
            bookmark_menu(menu, &view, id, window, cx)
        })
    }
}

/// An icon button that stays highlighted while its menu is open.
#[derive(IntoElement)]
struct MoreButton {
    base: Stateful<Div>,
    open: bool,
    open_bg: Hsla,
}

impl Selectable for MoreButton {
    fn selected(mut self, selected: bool) -> Self {
        self.open = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.open
    }
}

impl Styled for MoreButton {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for MoreButton {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl DropdownMenu for MoreButton {}

impl RenderOnce for MoreButton {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let bg = self.open_bg;
        if self.open {
            self.base.bg(bg)
        } else {
            self.base
        }
    }
}
