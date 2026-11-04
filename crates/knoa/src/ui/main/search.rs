//! The floating search pill and the kind chips under it.

use gpui::{
    Context, FontWeight, InteractiveElement as _, IntoElement, KeyDownEvent, ParentElement as _,
    StatefulInteractiveElement as _, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_component::input::Input;
use knoa_core::{Kind, Place};

use super::{MainView, Overlay};
use crate::ui::{
    components::icon,
    theme::{MARGIN, Theme},
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
            Place::UpNext => Some("Up next".to_owned()),
            Place::Unsorted => Some("Unsorted".to_owned()),
            Place::NotOrganized => Some("Not organized".to_owned()),
            Place::SetAside => Some("Set aside".to_owned()),
            Place::Folder(id) => library
                .folders()
                .iter()
                .find(|f| f.id == id)
                .map(|f| f.name.clone()),
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
                        .child(format!("{total} found")),
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

    fn kind_chips(&self, cx: &mut Context<'_, MainView>) -> impl IntoElement {
        let theme = Theme::get(cx).clone();
        let library = self.library.read(cx);
        let current = library.kind();
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
        if counts.len() > 1 || current.is_some() {
            row = row.child(chip("kind-all", "All".into(), current.is_none()).on_click(
                cx.listener(|this, _, _, cx| {
                    this.library
                        .update(cx, |library, cx| library.set_kind(None, cx));
                }),
            ));
            for (kind, count) in counts {
                row = row.child(
                    chip(
                        kind.slug(),
                        format!("{} {count}", kind.plural()),
                        current == Some(kind),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let kind: Option<Kind> =
                            (this.library.read(cx).kind() != Some(kind)).then_some(kind);
                        this.library
                            .update(cx, |library, cx| library.set_kind(kind, cx));
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
                if self.search.read(cx).value().is_empty() {
                    self.cards_focus.focus(window, cx);
                } else {
                    self.search
                        .update(cx, |input, cx| input.set_value("", window, cx));
                    self.library
                        .update(cx, |library, cx| library.set_search("", cx));
                }
                cx.stop_propagation();
            }
            _ => {}
        }
    }
}
