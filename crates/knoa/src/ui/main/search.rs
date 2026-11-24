//! The floating search pill and the kind chips under it.

use gpui::{
    Context, FontWeight, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::input::Input;
use knoa_core::{Filter, Kind, Place};

use super::{MainView, Overlay};
use crate::{
    i18n::{self, t},
    state::search::{Search, Token},
    ui::{
        components::icon,
        theme::{MARGIN, Theme},
    },
};

pub const PILL_HEIGHT: f32 = 50.;
const PILL_MAX_WIDTH: f32 = 620.;
/// The pill, the chips under it and the space before the first card.
pub const SEARCH_AREA_HEIGHT: f32 = PILL_HEIGHT + 52.;

impl MainView {
    pub(super) fn render_search(
        &mut self,
        show_detail: bool,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let (left, right) = super::content::content_span(window, show_detail);
        let width = (right - left - 40.).min(PILL_MAX_WIDTH);
        let library = self.library.read(cx);
        let total = library.results().len();
        let scope = match library.place() {
            Place::Everything => None,
            Place::Gallery => Some(t!("main-place-gallery")),
            Place::Events => Some(t!("main-place-events")),
            Place::UpNext => Some(t!("main-place-up-next")),
            Place::Unsorted => Some(t!("main-place-unsorted")),
            Place::NotOrganized => Some(t!("main-place-not-organized")),
            Place::SetAside => Some(t!("main-place-set-aside")),
            Place::Trash => Some(t!("main-place-trash")),
            Place::Folder(id) => {
                let tree = library.folder_tree();
                tree.get(id).map(|_| tree.path_name(id))
            }
        };
        let searching = !library.search().is_empty();

        let pill = div()
            .id("search-pill")
            .key_context("SearchBar")
            .capture_key_down(cx.listener(Self::search_keys))
            .w(px(width))
            .h(px(PILL_HEIGHT))
            .px_4()
            .flex()
            .items_center()
            .gap_2p5()
            .rounded_full()
            .bg(theme.panel)
            .border_1()
            .border_color(theme.hairline)
            .shadow_md()
            .child(icon("search", px(17.), theme.muted))
            .when_some(scope, |s, scope| {
                s.child(
                    div()
                        .flex_none()
                        .px_2()
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .rounded_full()
                        .bg(theme.soft)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .child(scope),
                )
            })
            .child(
                div()
                    .flex_1()
                    .child(Input::new(&self.search).appearance(false).cleanable(false)),
            )
            .when(searching, |s| {
                s.child(
                    div()
                        .flex_none()
                        .text_xs()
                        .text_color(theme.muted)
                        .child(t!("main-search-found", count = total)),
                )
            })
            .child(
                div()
                    .id("view-options")
                    .flex_none()
                    .size(px(30.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .child(icon("sliders-horizontal", px(16.), theme.muted))
                    .on_click(
                        cx.listener(|this, _, _, cx| this.open_overlay(Overlay::ViewOptions, cx)),
                    ),
            );

        div()
            .absolute()
            .top(MARGIN)
            .left(px(left))
            .w(px(right - left))
            .flex()
            .flex_col()
            .items_center()
            .gap_2()
            .child(pill)
            .child(self.kind_chips(cx))
    }

    /// After a language switch, what inputs keep rather than draw: their placeholders, and
    /// the search's `kind:` filters, in the new language's names.
    pub(super) fn translate(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.translate_search(window, cx);
        if let Some(edits) = &self.tag_edits {
            edits.translate(window, cx);
        }
        if let Some(edit) = &self.heading_edit {
            edit.translate(window, cx);
        }
    }

    /// Tokens draw their labels each frame, so only the placeholder needs translating.
    fn translate_search(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.sync_placeholder(window, cx);
    }

    /// The one way the UI changes the search: the input shows its typed text, the
    /// placeholder shows only without tokens, and the cards scroll back to the top.
    pub(super) fn set_search(
        &mut self,
        search: Search,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        if self.search.read(cx).value() != search.typed.as_str() {
            let typed = search.typed.clone();
            self.search
                .update(cx, |input, cx| input.set_value(typed, window, cx));
        }
        self.library
            .update(cx, |library, cx| library.set_search(search, cx));
        self.sync_placeholder(window, cx);
        self.scroll.set_offset(gpui::point(px(0.), px(0.)));
    }

    /// Reads what was typed into the search, turning complete syntax words into tokens.
    pub(super) fn on_search_input(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let text = self.search.read(cx).value().to_string();
        let mut search = self.library.read(cx).search().clone();
        search.take_input(&text);
        self.set_search(search, window, cx);
    }

    /// Clears the typed text and every token; the place stays.
    pub(super) fn clear_search(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        self.set_search(Search::default(), window, cx);
    }

    fn sync_placeholder(&mut self, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let placeholder = if self.library.read(cx).search().tokens.is_empty() {
            t!("main-search-placeholder")
        } else {
            String::new()
        };
        self.search.update(cx, |input, cx| {
            input.set_placeholder(placeholder, window, cx);
        });
    }

    /// Adds a kind token, or takes it away when it is the kind already applied.
    fn toggle_kind(&mut self, kind: Kind, window: &mut Window, cx: &mut Context<'_, MainView>) {
        let mut search = self.library.read(cx).search().clone();
        if search.kind() == Some(kind) {
            search
                .tokens
                .retain(|t| *t != Token::Filter(Filter::Kind(kind)));
        } else {
            search.add(Token::Filter(Filter::Kind(kind)));
        }
        self.set_search(search, window, cx);
    }

    fn kind_chips(&self, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let library = self.library.read(cx);
        let current = library.search().kind();
        let counts = library.kind_counts().to_vec();
        let chip = |id: &'static str, label: String, active: bool| {
            div()
                .id(id)
                .h(px(28.))
                .px_3()
                .flex()
                .items_center()
                .rounded_full()
                .text_xs()
                .cursor_pointer()
                .font_weight(FontWeight::MEDIUM)
                .map(|s| {
                    if active {
                        s.bg(theme.text).text_color(theme.panel)
                    } else {
                        s.bg(theme.panel.opacity(0.7))
                            .text_color(theme.muted)
                            .hover(|s| s.bg(theme.panel))
                    }
                })
                .child(label)
        };
        let mut row = div().flex().flex_wrap().justify_center().gap_1p5();
        // The Gallery holds media kinds only, so choosing among them says little.
        let in_gallery = library.place() == Place::Gallery;
        if !in_gallery && (counts.len() > 1 || current.is_some()) {
            row = row.child(
                chip("kind-all", t!("main-kind-all"), current.is_none()).on_click(cx.listener(
                    move |this, _, window, cx| {
                        if let Some(kind) = current {
                            this.toggle_kind(kind, window, cx);
                        }
                    },
                )),
            );
            for (kind, count) in counts {
                row = row.child(
                    chip(
                        kind.slug(),
                        t!(
                            "main-kind-chip",
                            kind = (i18n::kind_plural(kind)),
                            count = count
                        ),
                        current == Some(kind),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.toggle_kind(kind, window, cx);
                    })),
                );
            }
        }
        row
    }

    /// Escape and ↓ leave the search field for the cards.
    fn search_keys(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<'_, MainView>,
    ) {
        match event.keystroke.key.as_str() {
            "down" => {
                self.select_first(cx);
                self.cards_focus.focus(window, cx);
                cx.stop_propagation();
            }
            "escape" => {
                if self.library.read(cx).search().is_empty() {
                    self.cards_focus.focus(window, cx);
                } else {
                    self.clear_search(window, cx);
                }
                cx.stop_propagation();
            }
            _ => {}
        }
    }
}
