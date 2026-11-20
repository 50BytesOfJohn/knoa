//! A button that opens a dropdown menu and stays highlighted while the menu is open.

use gpui::{
    App, Div, Hsla, InteractiveElement, Interactivity, IntoElement, RenderOnce, Stateful,
    StyleRefinement, Styled, Window,
};
use gpui_component::{Selectable, menu::DropdownMenu};

#[derive(IntoElement)]
pub(in crate::ui) struct MenuTrigger {
    base: Stateful<Div>,
    open: bool,
    open_bg: Hsla,
}

impl MenuTrigger {
    /// `base` drawn with `open_bg` behind it while its menu shows.
    pub fn new(base: Stateful<Div>, open_bg: Hsla) -> MenuTrigger {
        MenuTrigger {
            base,
            open: false,
            open_bg,
        }
    }
}

impl Selectable for MenuTrigger {
    fn selected(mut self, selected: bool) -> Self {
        self.open = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.open
    }
}

impl Styled for MenuTrigger {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for MenuTrigger {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl DropdownMenu for MenuTrigger {}

impl RenderOnce for MenuTrigger {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let bg = self.open_bg;
        if self.open {
            self.base.bg(bg)
        } else {
            self.base
        }
    }
}
