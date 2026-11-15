//! A card at its place on the canvas, and what clicking a card does.

use gpui::{
    ClickEvent, Context, IntoElement, ParentElement as _, StatefulInteractiveElement as _,
    Styled as _, Window, div, px,
};
use knoa_browser::Activation;
use knoa_core::{Bookmark, BookmarkId};

use crate::ui::{
    main::{
        MainView, bookmark_menu::with_bookmark_menu, cards, content::Arrangement, masonry::Placed,
        tiles,
    },
    theme::Theme,
};

impl MainView {
    pub(super) fn placed_card(
        &self,
        bookmark: &Bookmark,
        placed: &Placed,
        show_summaries: bool,
        theme: &Theme,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let id = bookmark.id;
        let card = match self.layout.arrangement {
            Arrangement::Cards => cards::render(bookmark, placed.width, show_summaries, cx),
            Arrangement::Pictures => {
                let selected = self.selected == Some(id);
                tiles::render(bookmark, placed.width, selected, cx)
            }
        };
        let on_click = move |this: &mut MainView,
                             event: &ClickEvent,
                             window: &mut Window,
                             cx: &mut Context<'_, MainView>| {
            this.click_card(id, event, window, cx);
        };
        div()
            .absolute()
            .top(px(placed.y))
            .left(px(placed.x))
            .w(px(placed.width))
            .child(with_bookmark_menu(
                cards::interactive(placed.index, card, placed.height, theme)
                    .on_click(cx.listener(on_click))
                    .on_aux_click(cx.listener(on_click))
                    .child(
                        cards::open_button(placed.index, theme).on_click(cx.listener(
                            move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.open_bookmark(id, Activation::Foreground, cx);
                            },
                        )),
                    ),
                id,
                cx,
            ))
    }

    pub(super) fn click_card(
        &mut self,
        id: BookmarkId,
        event: &ClickEvent,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        self.cards_focus.focus(window, cx);
        if let Some(activation) = new_tab_activation(event) {
            self.open_bookmark(id, activation, cx);
        } else if event.standard_click() && event.click_count() >= 2 {
            // In the Gallery the picture is the bookmark, so it opens in the viewer.
            if self.layout.arrangement == Arrangement::Pictures && self.picture_of(id, cx).is_some()
            {
                self.open_viewer(id, window, cx);
            } else {
                self.open_bookmark(id, Activation::Foreground, cx);
            }
        } else if event.standard_click() && self.layout.arrangement == Arrangement::Pictures {
            // Opening the panel would reflow the grid under a double-click's second click.
            // An open panel follows the selection; Cmd-I or the menu opens it.
            self.set_selected(Some(id), cx);
            cx.notify();
        } else if event.standard_click() {
            self.select(id, cx);
        }
    }
}

/// Clicks that open a link in a new tab in a browser open the bookmark straight away:
/// Cmd-click and middle-click leave Knoa in front, and adding Shift brings the browser up.
fn new_tab_activation(event: &ClickEvent) -> Option<Activation> {
    let modifiers = event.modifiers();
    let new_tab_click = event.is_middle_click() || (event.standard_click() && modifiers.platform);
    if !new_tab_click {
        return None;
    }
    Some(if modifiers.shift {
        Activation::Foreground
    } else {
        Activation::Background
    })
}
